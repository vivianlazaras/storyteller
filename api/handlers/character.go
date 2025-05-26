package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterUser(r *gin.Engine) *gin.Engine {
	r.GET("/characters", ListPubCharacters)
    r.GET("/characters/:id", GetCharacter)
    r.POST("/characters", CreateCharacter)
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
type CreateCharacter struct {
	Name string			`json:"name"`
	Description *string	`json:"description"`
	Image *string		`json:"image"`
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

func CreateCharacterFromFragment(fragment *CreateCharacter, creatorID uuid.UUID) {
	now := time.Now().Unix()
	description := ""
	if fragment.Description != nil {
		description = *fragment.Description
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Character{}, err
	}

	timeline, err := createDefaultTimeline(metadata.ID)
	if err != nil {
		return model.Character{}, err
	}

	var character = model.Character{
		ID:          uuid.New().String(),
		Metadata:	 metadata.ID,
		Timeline:    timeline.ID,
		Name:        fragment.Name,
		Description: description,
		Image:		 fragment.Image,
		Created:     now,
		LastEdited:  now,
	}

	dberr := db.DB.Create(&story).Error
	return story, dberr
}

func CreateCharacter(c *gin.Context) {
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := getUserByEmail("vivianlazaras@gmail.com")

	var fragment CreateCharacter
	if err := c.ShouldBindJSON(&fragment); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	character, err := CreateStoryFromFragment(&fragment, parsedUUID)
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
