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
)

func RegisterCharacterRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/characters", ListPubCharacters)
    r.GET("/characters/:id", auth.JWTMiddleware(), GetCharacter)
    r.POST("/characters", auth.JWTMiddleware(), CreateCharacter)
	r.GET("/characters/filter", auth.JWTMiddleware(), GetCharactersByStory)
    /*r.PUT("/characters/:id", middleware.RequireOIDC(), UpdateCharacter)
    r.DELETE("/characters/:id", middleware.RequireOIDC(), DeleteCharacter)
	*/return r
}

// for now this route can only fetch public characters
func ListPubCharacters(c *gin.Context) {
	
	var characters []model.Character
    err := db.DB.
        Where("metadata.public = ?", true).
        Joins("JOIN metadata ON metadata.id = characters.metadata").
        Find(&characters).Error
	
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, characters)
}
// hmm, when a character that isn't published is linked to a published story
// what should happen? Should the character get auto published? How about auto shared in groups?
type CharacterBuilder struct {
	Name string			`json:"name"`
	Description *string	`json:"description"`
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

	c.JSON(http.StatusOK, character)
}

func CreateCharacterFromFragment(fragment *CharacterBuilder, creatorID uuid.UUID) (model.Character, error) {
	now := time.Now().Unix()
	description := ""
	if fragment.Description != nil {
		description = *fragment.Description
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Character{}, err
	}

	var character = model.Character{
		ID:          uuid.New().String(),
		Metadata:	 metadata.ID,
		Name:        fragment.Name,
		Description: description,
		Created:     now,
		LastEdited:  now,
	}

	dberr := db.DB.Create(&character).Error
	return character, dberr
}

func CreateCharacter(c *gin.Context) {
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := auth.GetUserFromClaims(db.DB, c)

	var fragment CharacterBuilder
	if err := c.ShouldBindJSON(&fragment); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	character, err := CreateCharacterFromFragment(&fragment, parsedUUID)
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

	c.JSON(http.StatusOK, characters)
}