package handlers

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/middleware"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func RegisterStoryRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/stories", ListPubStories)
    r.GET("/stories/:id", GetStory)
    r.POST("/stories", CreateStory)
    r.PUT("/stories/:id", middleware.RequireOIDC(), UpdateStory)
    r.DELETE("/stories/:id", middleware.RequireOIDC(), DeleteStory)
	return r
}

func ListPubStories(c *gin.Context) {
	// grab all stories where public = true
	
	c.JSON(http.StatusOK, []model.Story{})
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

func CreateStoryFromFragment(fragment CreateStoryFragment, creatorID) (model.Story, error) {
	now := time.Now().Unix()
	description := ""
	if fragment.Description != nil {
		description = *fragment.Description
	}

	// have to create a timeline, and metadata
	var story = model.Story{
		ID:          uuid.New().String(),
		Timeline:    timelineID,
		Name:        fragment.Name,
		Description: description,
		Content:     []byte(fragment.Content),
		Created:     now,
		LastEdited:  now,
		Renderer:    string(fragment.Render),
	}

	err := db.DB.Create(story).Error
	return story, err
}

func CreateStory(c *gin.Context) {
	var fragment CreateStoryFragment
	if err := c.ShouldBindJSON(&fragment); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}


}

func UpdateStory(c *gin.Context) {

}

func DeleteStory(c *gin.Context) {
	
}