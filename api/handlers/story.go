package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/middleware"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterStoryRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/stories", ListPubStories)
    r.GET("/stories/:id", GetStory)
    r.POST("/stories", middleware.RequireOIDC(), CreateUser)
    r.PUT("/stories/:id", middleware.RequireOIDC(), UpdateUser)
    r.DELETE("/stories/:id", middleware.RequireOIDC(), DeleteUser)
	return r
}

func ListPubStories(c *gin.Context) {
	// grab all stories where public = true
	
	c.JSON(http.StatusOK, []model.Story{})
}

func GetStory(c *gin.Context) {
	story, err := db.GetByID[model.Story](c, "stories");
	if err != nil {
		return
	}

	if story.Public != true {
		c.JSON(http.StatusNotFound, model.Story{})
		return
	}

	c.JSON(http.StatusOK, story)
}

func CreateStory(c *gin.Context) {

}

func UpdateStory(c *gin.Context) {

}

func DeleteStory(c *gin.Context) {
	
}