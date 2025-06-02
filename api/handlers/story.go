package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	// "github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func RegisterStoryRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/stories", ListPubStories)
	r.GET("/stories/fragments/:id", GetFragmentById)
    r.GET("/stories/:id", GetStory)
	r.GET("/stories/fragments", GetFragmentsByStory)
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

func GetFragmentsByStory(c *gin.Context) {
	IDString := c.Query("story")
	storyID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse story as UUID"})
		return
	}

	var fragments []model.Fragment
	if err := db.DB.Where("story = ?", storyID).Find(&fragments).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, fragments)
}

func GetFragmentById(c *gin.Context) {
	fragment, err := db.GetByCtxID[model.Fragment](c, "fragments");
	if err != nil {
		return
	}

	// get fragments, characters, places
	c.JSON(http.StatusOK, fragment)
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

type CreateStoryFragment struct {
	Name        string          `json:"name"`
	Description *string         `json:"description,omitempty"`
	Render      string 			`json:"render"`
	Content     string          `json:"content"`
	Image		*string			`json:"image"`
}

func CreateStoryFromFragment(fragment *CreateStoryFragment, creatorID uuid.UUID) (model.Fragment, error) {
	now := time.Now().Unix()
	description := ""
	image		:= ""
	if fragment.Description != nil {
		description = *fragment.Description
	}
	if fragment.Image != nil {
		image = *fragment.Image
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Fragment{}, err
	}

	timeline, err := createDefaultTimeline(metadata.ID)
	if err != nil {
		return model.Fragment{}, err
	}

	var storyid = uuid.New().String();

	var story = model.Story {
		ID:          storyid,
		Metadata:	 metadata.ID,
		Timeline:    timeline.ID,
		Name:        fragment.Name,
		Description: description,
		Image:		 image,
		Created:     now,
		LastEdited:  now,
		Renderer:    string(fragment.Render),
	}

	dberr := db.DB.Create(&story).Error
	if dberr != nil {
		return model.Fragment{}, dberr
	}

	var newfragment = model.Fragment {
		ID: 		uuid.New().String(),
		Story: 		storyid,
		Metadata:	metadata.ID,
		Content:	fragment.Content,
		Name:		fragment.Name,
		Image:		image,
		LastEdited:	now,
		Created:	now,
	}

	fragmentdberr := db.DB.Create(&newfragment).Error
	return newfragment, fragmentdberr
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