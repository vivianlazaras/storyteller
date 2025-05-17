package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func RegisterTimelineRoutes(r *gin.Engine) *gin.Engine {
	// after testing I may not expose these, opting rather to handle interaction
	// with timelines through stories, or characters
	r.GET("/stories", ListTimelines)
    r.GET("/stories/:id", GetTimeline)
	return r
}

func ListTimelines(c *gin.Context) {
	// grab all stories where public = true
	c.JSON(http.StatusOK, []model.Story{})
}

func GetTimeline(c *gin.Context) {
	timeline, err := db.GetByCtxID[model.Timeline](c, "timelines")
	if err != nil {
		return
	}

	//if character.Public != true {
	//	c.JSON(http.StatusNotFound, model.Timeline{})
	//	return
	//}

	c.JSON(http.StatusOK, timeline)
}

func CreateTimeline(metadataID uuid.UUID) (model.Timeline, error) {
	now := time.Now().Unix()

	var timeline = model.Timeline{
		ID:         uuid.New().String(),
		Created:    now,
		LastEdited: now,
		Metadata: metadataID.String(),
	}

	err := db.DB.Create(timeline).Error
	return timeline, err
}

func UpdateTimeline(c *gin.Context) {

}

func DeleteTimeline(c *gin.Context) {
	
}

