package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/google/uuid"	
)

type FullMoment struct {
	ID			uuid.UUID `json:"id"`
	TimeLine	uuid.UUID `json:"timeline"`
	Fragment	model.Fragment	`json:"fragment"`
	Idx			int64	`json:"idx"`
}

type FullTimeline struct {
	ID			uuid.UUID 	`json:"id"`
	Name		string	`json:"name"`
	Description	string	`json:"description"`
	Created		int64	`json:"created"`
	Moments		[]FullMoment	`json:"moments"`
	Graph		*string	`json:"graph"`
}

func RegisterTimelineRoutes(r *gin.Engine) *gin.Engine {
	// after testing I may not expose these, opting rather to handle interaction
	// with timelines through stories, or characters
	r.GET("/timelines", auth.JWTMiddleware(), ListTimelines)
    r.GET("/timelines/:id", auth.JWTMiddleware(), GetTimeline)

	return r
}

func ListTimelines(c *gin.Context) {
	// grab all stories where public = true
	c.JSON(http.StatusOK, []model.Story{})
}

func GetTimeline(c *gin.Context) {
	// Get the timeline by ID from context (assuming from URL param or similar)
	timeline, err := db.GetByCtxID[model.Timeline](c, "timelines")
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "timeline not found"})
		return
	}

	// Fetch the associated moments
	var moments []model.Moment
	if err := db.DB.
		Where("timeline = ?", timeline.ID).
		Order("idx ASC").
		Find(&moments).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to retrieve moments"})
		return
	}

	// Compose FullMoments by fetching their fragments
	fullMoments := make([]FullMoment, 0, len(moments))
	for _, moment := range moments {
		var fragment model.Fragment
		if err := db.DB.
			Where("id = ?", moment.Fragment).
			First(&fragment).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to retrieve fragment"})
			return
		}

		fullMoments = append(fullMoments, FullMoment{
			ID:       uuid.MustParse(moment.ID),
			TimeLine: uuid.MustParse(moment.Timeline),
			Fragment: fragment,
			Idx:      moment.Idx,
		})
	}

	// Attempt to fetch the graph, if any
	var graph model.Graph
	var graphStr *string
	if err := db.DB.
		Where("entity = ?", timeline.ID).
		Order("rendered DESC").
		First(&graph).Error; err == nil {
		graphStr = &graph.DotData
	}

	// Build and return the FullTimeline
	full := FullTimeline{
		ID:          uuid.MustParse(timeline.ID),
		Name:        timeline.Name,
		Description: timeline.Description,
		Created:     timeline.Created,
		Moments:     fullMoments,
		Graph:       graphStr,
	}

	c.JSON(http.StatusOK, full)
}

func defaultTimeline(metadata string) model.Timeline {
	now := time.Now().Unix()

	return model.Timeline{
		ID:         uuid.New().String(),
		Created:    now,
		LastEdited: now,
		Metadata: metadata,
	}

}

func createTimeline(timeline *model.Timeline) error {
	err := db.DB.Create(timeline).Error
	return err
}

func createDefaultTimeline(metadata string) (model.Timeline, error) {
	var timeline = defaultTimeline(metadata)
	err := createTimeline(&timeline)
	return timeline, err
}

func UpdateTimeline(c *gin.Context) {

}

func DeleteTimeline(c *gin.Context) {
	
}

