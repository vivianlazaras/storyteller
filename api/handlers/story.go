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

	// get fragments, characters, places
	c.JSON(http.StatusOK, story)
}

type CreateStoryParts struct {
	Title        string          `json:"title"`
	Description *string         `json:"description,omitempty"`
	Render      string 			`json:"render"`
	Image		*string			`json:"image"`
	Tags		[]string		`json:"tags"`
}

func CreateStoryFromParts(fragment *CreateStoryParts, creatorID uuid.UUID) (model.Story, error) {
	now := time.Now().Unix()
	description := ""
	image		:= ""
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

	var storyid = uuid.New();

	var story = model.Story {
		ID:          storyid.String(),
		Metadata:	 metadata.ID,
		Timeline:    timeline.ID,
		Name:        fragment.Title,
		Description: description,
		Image:		 image,
		Created:     now,
		LastEdited:  now,
		Renderer:    string(fragment.Render),
	}

	dberr := db.DB.Create(&story).Error
	if dberr != nil {
		return model.Story{}, dberr
	}

	tagerr := InsertTagsForEntity(storyid, fragment.Tags)
	
	return story, tagerr
}

func CreateStory(c *gin.Context) {
	
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := getUserByEmail("vivianlazaras@gmail.com")

	var parts CreateStoryParts
	if err := c.ShouldBindJSON(&parts); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	story, err := CreateStoryFromParts(&parts, parsedUUID)
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