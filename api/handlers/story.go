package handlers

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	// "github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func RegisterStoryRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/stories", ListPubStories)
    r.GET("/stories/:id", GetStory)
    r.POST("/stories", CreateStory)
	return r
}

func ListPubStories(c *gin.Context) {
	// grab all stories where public = true
	var stories []model.Story
    err := db.DB.
        Where("metadata.public = ?", true).
        Joins("JOIN metadata ON metadata.id = stories.metadata").
        Find(&stories).Error
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, stories)
}

func GetStory(c *gin.Context) {
	story, err := db.GetByCtxID[model.Story](c, "stories");
	if err != nil {
		return
	}

	metadata, err := db.GetByID[model.Metadatum]("metadata", story.Metadata)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to fetch metadata"})
	}
	if metadata.Public != true {
		c.JSON(http.StatusNotFound, model.Story{})
		return
	}

	c.JSON(http.StatusOK, story)
}

type CreateStoryFragment struct {
	Name        string          `json:"name"`
	Description *string         `json:"description,omitempty"`
	Render      string 			`json:"render"`
	Content     string          `json:"content"`
}

func CreateStoryFromFragment(fragment *CreateStoryFragment, creatorID uuid.UUID) (model.Story, error) {
	now := time.Now().Unix()
	description := ""
	if fragment.Description != nil {
		description = *fragment.Description
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Story{}, err
	}

	timeline, err := createDefaultTimeline(metadata.ID)
	if err != nil {
		return model.Story{}, err
	}

	var story = model.Story{
		ID:          uuid.New().String(),
		Metadata:	 metadata.ID,
		Timeline:    timeline.ID,
		Name:        fragment.Name,
		Description: description,
		Content:     fragment.Content,
		Created:     now,
		LastEdited:  now,
		Renderer:    string(fragment.Render),
	}

	dberr := db.DB.Create(&story).Error
	return story, dberr
}

func CreateStory(c *gin.Context) {
	
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := getUserByEmail("vivianlazaras@gmail.com")

	var fragment CreateStoryFragment
	if err := c.ShouldBindJSON(&fragment); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	story, err := CreateStoryFromFragment(&fragment, parsedUUID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{ "error": "Internal Server Error: " + err.Error() })
		return
	}

	c.JSON(http.StatusOK, story)
}

func UpdateStory(c *gin.Context) {

}

func DeleteStory(c *gin.Context) {
	
}