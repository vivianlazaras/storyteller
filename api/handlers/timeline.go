package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/middleware"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterTimelineRoutes(r *gin.Engine) *gin.Engine {
	// after testing I may not expose these, opting rather to handle interaction
	// with timelines through stories, or characters
	r.GET("/stories", ListTimelines)
    r.GET("/stories/:id", GetTimeline)
    r.POST("/stories", middleware.RequireOIDC(), CreateTimeline)
    r.PUT("/stories/:id", middleware.RequireOIDC(), UpdateTimeline)
    r.DELETE("/stories/:id", middleware.RequireOIDC(), DeleteTimeline)
	return r
}

func ListTimelines(c *gin.Context) {
	// grab all stories where public = true
	c.JSON(http.StatusOK, []model.Story{})
}

func GetTimeline(c *gin.Context) {
	timeline, err := db.GetByID[model.Timeline](c, "timelines")
	if err != nil {
		return
	}

	//if character.Public != true {
	//	c.JSON(http.StatusNotFound, model.Timeline{})
	//	return
	//}

	c.JSON(http.StatusOK, timeline)
}

func CreateTimeline(c *gin.Context) {

}

func UpdateTimeline(c *gin.Context) {

}

func DeleteTimeline(c *gin.Context) {
	
}

