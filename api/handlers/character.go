package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/google/uuid"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"gorm.io/gorm"
)

func RegisterCharacterRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/characters", auth.JWTMiddleware(), ListCharacters)
    r.GET("/characters/:id", auth.JWTMiddleware(), GetCharacter)
    r.POST("/characters", auth.JWTMiddleware(), CreateCharacter)
	r.GET("/characters/filter", auth.JWTMiddleware(), GetCharactersByStory)
	r.GET("/characters/tree", auth.JWTMiddleware(), GetCharacterTree)
    //r.PUT("/characters/:id", middleware.RequireOIDC(), UpdateCharacter)
    r.DELETE("/characters/:id", auth.JWTMiddleware(), DeleteCharacter)
	return r
}

type CharacterRender struct {
	Thumbnail		*model.Image	`json:"thumbnail"`
	Tags			[]model.Tag	`json:"tags"`
	ID				uuid.UUID	`json:"id"`
	Name			string		`json:"name"`
	Description		*string		`json:"description"`
	Images			[]model.Image	`json:"images"`
}

func renderCharacter(tx *gorm.DB, character model.Character) (CharacterRender, error) {
	var charid = uuid.MustParse(character.ID)
	tags, tagerr := SelectTagsByEntityID(charid)
	if tagerr != nil {
		return CharacterRender{}, nil
	}
	
	thumbnail, thmerr := db.GetByID[model.Image]("images", character.Thumbnail);
	if thmerr != nil {
		return CharacterRender{}, thmerr
	}

	images, imgerr := GetImagesByParentID(tx, charid)
	if imgerr != nil {
		return CharacterRender{}, imgerr
	}

	return CharacterRender{
		Thumbnail: thumbnail,
		Tags:  tags,
		ID: charid,
		Name: character.Name,
		Description: &character.Description,
		Images: images,
	}, nil
}

// for now this route can only fetch public characters
func ListCharacters(c *gin.Context) {
	
	var characters 	[]model.Character
	var renders		[]CharacterRender
    err := db.DB.
        Where("metadata.public = ?", true).
        Joins("JOIN metadata ON metadata.id = characters.metadata").
        Find(&characters).Error
	
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}

	for _, character := range characters {
		render, renderr := renderCharacter(db.DB, character)
		if renderr != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": renderr})
		}
		renders = append(renders, render)
		
	}
	c.JSON(http.StatusOK, renders)
}
// hmm, when a character that isn't published is linked to a published story
// what should happen? Should the character get auto published? How about auto shared in groups?
type CharacterBuilder struct {
	Name string			`json:"name"`
	Description *string	`json:"description"`
	Thumbnail		*ImageBuilder
}

func GetCharacter(c *gin.Context) {
	character, err := db.GetByCtxID[model.Character](c, "characters")
	if err != nil {
		return
	}

	metadata, err := db.GetByID[model.Metadatum]("metadata", character.Metadata)
	if metadata.Public != true {
		c.JSON(http.StatusNotFound, model.Character{})
		return
	}

	render, renderr := renderCharacter(db.DB, *character)

	if renderr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": renderr})
	}
	c.JSON(http.StatusOK, render)
}

func CreateNewCharacter(tx *gorm.DB, builder *CharacterBuilder, creatorID uuid.UUID) (model.Character, error) {
	now := time.Now().Unix()
	description := ""
	if builder.Description != nil {
		description = *builder.Description
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Character{}, err
	}

	var character = model.Character{
		ID:          uuid.New().String(),
		Metadata:	 metadata.ID,
		Name:        builder.Name,
		Description: description,
		Created:     now,
		LastEdited:  now,
	}

	if builder.Thumbnail != nil {
		_, imgerr := CreateNewImage(tx, *builder.Thumbnail)
		if imgerr != nil {
			return model.Character{}, imgerr
		}
	}

	dberr := tx.Create(&character).Error
	return character, dberr
}

func CreateCharacter(c *gin.Context) {
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := auth.GetUserFromClaims(db.DB, c)

	var builder CharacterBuilder
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	character, err := CreateNewCharacter(db.DB, &builder, parsedUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{ "error": "Internal Server Error: " + err.Error() })
		return
	}

	c.JSON(http.StatusOK, character)
}

func UpdateCharacter(c *gin.Context) {
	c.JSON(http.StatusOK, model.Character{})
}

func DeleteCharacter(c *gin.Context) {
	c.JSON(http.StatusOK, model.Character{})
}

func GetCharactersByStory(c *gin.Context) {
	IDString := c.Query("parent")
	parentID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse parent as UUID"})
		return
	}

	var characters []model.Character
	err := db.DB.
		Model(&model.Character{}).
		Joins("JOIN relations ON relations.child = characters.id").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, "stories", "characters").
		Find(&characters).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	characterRenders := make([]CharacterRender, 0, len(characters))

	for _, character := range characters {
		var thumbnail *model.Image
		if character.Thumbnail != "" {
			var img model.Image
			if err := db.DB.First(&img, "id = ?", character.Thumbnail).Error; err == nil {
				thumbnail = &img
			} else {
				fmt.Printf("warning: failed to load thumbnail for location %s: %v\n", character.ID, err)
			}
		}

		render := CharacterRender{
			ID:          uuid.MustParse(character.ID),
			Name:        character.Name,
			Description: &character.Description,
			Thumbnail:   thumbnail,
		}
		characterRenders = append(characterRenders, render)
	}

	c.JSON(http.StatusOK, characterRenders)
}

func GetCharacterTree(c *gin.Context) {
	parentID, iderr := uuid.Parse(c.Query("id"))
	if iderr != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "couldn't parse ID query param"})
		return
	}
	var visited = make(map[uuid.UUID]bool)
	root, err := FetchCharacterTree(db.DB, parentID, visited)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, root)
}

type CharacterNode struct {
	ID          uuid.UUID       `json:"id"`
	Name        string          `json:"name"`
	Description string          `json:"description"`
	Children    []CharacterNode `json:"children"`
}

func FetchCharacterTree(db *gorm.DB, parentID uuid.UUID, visited map[uuid.UUID]bool) (CharacterNode, error) {
	// Prevent cycles
	if visited[parentID] {
		return CharacterNode{}, nil
	}
	visited[parentID] = true

	var character model.Character
	err := db.First(&character, "id = ?", parentID).Error
	if err != nil {
		return CharacterNode{}, fmt.Errorf("error fetching character: %w", err)
	}

	node := CharacterNode{
		ID:          uuid.MustParse(character.ID),
		Name:        character.Name,
		Description: character.Description,
		Children:    []CharacterNode{},
	}

	// Fetch child IDs
	var childIDs []uuid.UUID
	err = db.Model(&model.Relation{}).
		Where("parent = ? AND child_category = ?", parentID, "characters").
		Pluck("child", &childIDs).Error
	if err != nil {
		return node, fmt.Errorf("error fetching children: %w", err)
	}

	// Recursively build child nodes
	for _, childID := range childIDs {
		childNode, err := FetchCharacterTree(db, childID, visited)
		if err != nil {
			return node, err
		}
		if childNode.ID != uuid.Nil {
			node.Children = append(node.Children, childNode)
		}
	}

	return node, nil
}