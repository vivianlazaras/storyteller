package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
	"gorm.io/gorm"
	"fmt"
)

func RegisterStoryRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/stories", auth.JWTMiddleware(), GetStories)
    r.GET("/stories/:id", auth.JWTMiddleware(), GetStory)
    r.POST("/stories", auth.JWTMiddleware(), CreateStory)
	r.DELETE("/stories/:id", auth.JWTMiddleware(), DeleteStory)
	return r
}

func GetStories(c *gin.Context) {
	user, usererr := auth.GetUserFromClaims(db.DB, c)
	if usererr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "could not retrieve user"})
		return
	}

	// grab all stories for the user's default group
	var stories []model.Story
    err := db.DB.
        Where("entities.active = ? and entities.group_id = ?", true, user.DefaultGroup).
        Joins("JOIN entities ON entities.id = stories.id").
		Order("stories.last_edited DESC").
        Find(&stories).Error
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, stories)
}

func GetStory(c *gin.Context) {
	
	/*user, usererr := auth.GetUserFromClaims(db.DB, c)
	if usererr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "could not retrieve user"})
		return
	}*/
	
	fmt.Println("able to get user info")
	story, err := GetByCtxID[model.Story](db.DB, c, "stories");
	if err != nil {
		return
	}

	fmt.Println("able to get story info")
	fmt.Println("able to get metadata")

	// get fragments, characters, places
	c.JSON(http.StatusOK, story)
}

type StoryBuilder struct {
	Title        string          `json:"title"`
	Description *string         `json:"description,omitempty"`
	Render      string 			`json:"render"`
	Image		*string			`json:"image"`
	Tags		[]string		`json:"tags"`
	Group		*string			`json:"group"`
}

func CreateNewStory(tx *gorm.DB, fragment *StoryBuilder, userID, groupID uuid.UUID) (model.Story, error) {
	now := time.Now().Unix()
	

	metadata, err := createDefaultMetadata(userID)
	if err != nil {
		return model.Story{}, err
	}

	var storyid = uuid.New();
	var fragment_render = string(fragment.Render)
	var story = model.Story {
		ID:          storyid,
		Metadata:	 &metadata.ID,
		Name:        fragment.Title,
		Description: fragment.Description,
		Image:		 fragment.Image,
		Created:     now,
		LastEdited:  &now,
		Renderer:    &fragment_render,
	}

	dberr := tx.Create(&story).Error
	if dberr != nil {
		return model.Story{}, dberr
	}

	// this will also check to ensure the user has access to the group, so that logic is in one place
	if err := CreateNewEntity(tx, storyid, userID, groupID); err != nil {
		return model.Story{}, err
	}

	tagerr := InsertTagsForEntity(tx, storyid, fragment.Tags)
	
	return story, tagerr
}

func CreateStory(c *gin.Context) {
	// Handle user retrieval from JWT claims
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized: " + err.Error()})
		return
	}

	var parts StoryBuilder
	if err := c.ShouldBindJSON(&parts); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid user ID"})
		return
	}

	// Begin transaction
	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create transaction"})
		return
	}

	// Create new story
	story, err := CreateNewStory(tx, &parts, user.ID, user.DefaultGroup)
	if err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Internal Server Error: " + err.Error()})
		return
	}

	// Update group_id in entities table for the created story
	if err := tx.Model(&model.Entity{}).
		Where("id = ?", story.ID).
		Update("group_id", user.DefaultGroup).Error; err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update group_id: " + err.Error()})
		return
	}

	// Commit transaction
	if err := tx.Commit().Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to commit transaction"})
		return
	}

	c.JSON(http.StatusOK, story)
}

func UpdateStory(c *gin.Context) {

}

func DeleteStory(c *gin.Context) {
	id := c.Param("id")

	var story model.Story
	if err := db.DB.First(&story, "id = ?", id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Story not found"})
		return
	}

	// Update metadata.active = false
	if err := db.DB.Model(&model.Metadatum{}).
		Where("id = ?", story.Metadata).
		Update("active", false).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to deactivate story"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Story deactivated successfully"})
}