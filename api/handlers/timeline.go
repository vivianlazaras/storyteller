package handlers

import (
	"net/http"
	"time"
	"fmt"
	"encoding/json"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type TimelineBuilder struct {
	Name        string              `json:"name"`
	Description *string             `json:"description,omitempty"`
	Source     	uuid.UUID	`json:"source"`
	Category	string	`json:"category"`
}

type FullMoment struct {
	ID			uuid.UUID `json:"id"`
	Timeline	uuid.UUID `json:"timeline"`
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
	r.POST("/timelines", auth.JWTMiddleware(), CreateTimeline)

	return r
}

func GetTimeline(c *gin.Context) {
	// Get the timeline by ID from context (assuming from URL param or similar)
	timeline, err := db.GetByCtxID[model.Timeline](c, "timelines")
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "timeline not found"})
		return
	}
	// I need to find out how to automatically re-generate if updates have occured.

	full, err := GetFullTimeline(db.DB, *timeline);

	c.JSON(http.StatusOK, full)
}

func GetFullTimeline(db *gorm.DB, timeline model.Timeline) (FullTimeline, error) {
	// Fetch the associated moments
	var moments []model.Moment
	if err := db.
		Where("timeline = ?", timeline.ID).
		Order("idx ASC").
		Find(&moments).Error; err != nil {
		return FullTimeline{}, err
	}

	// Compose FullMoments by fetching their fragments
	fullMoments := make([]FullMoment, 0, len(moments))
	for _, moment := range moments {
		var fragment model.Fragment
		if err := db.
			Where("id = ?", moment.Fragment).
			First(&fragment).Error; err != nil {
			return FullTimeline{}, err
		}

		fullMoments = append(fullMoments, FullMoment{
			ID:       uuid.MustParse(moment.ID),
			Timeline: uuid.MustParse(moment.Timeline),
			Fragment: fragment,
			Idx:      moment.Idx,
		})
	}

	// Attempt to fetch the graph, if any
	var graph model.Graph
	var graphStr *string
	if err := db.
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

	return full, nil
}

func defaultTimeline(metadata string) model.Timeline {
	now := time.Now().Unix()

	return model.Timeline{
		ID:         uuid.New().String(),
		Created:    now,
		LastEdited: now,
	}

}
/*
func createTimeline(timeline *model.Timeline) error {
	err := db.DB.Create(timeline).Error
	return err
}

func createDefaultTimeline(metadata string) (model.Timeline, error) {
	var timeline = defaultTimeline(metadata)
	err := createTimeline(&timeline)
	return timeline, err
}*/

func UpdateTimeline(c *gin.Context) {

}

func DeleteTimeline(c *gin.Context) {
	
}

func CreateMoment(db *gorm.DB, timeline uuid.UUID, fragment uuid.UUID, idx int64) (model.Moment, error) {
	var moment = model.Moment {
		ID: uuid.New().String(),
		Timeline: timeline.String(),
		Fragment: fragment.String(),
		Idx: idx * 8,
	}

	err := db.Create(&moment)
	return moment, err.Error
}

func ListTimelines(c *gin.Context) {
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "unauthorized: " + err.Error()})
		return
	}

	// Fetch timelines accessible by this user
	timelines, err := ListEntitiesByCategoryForGroup(db.DB, uuid.MustParse(user.DefaultGroup), "timelines")
	
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to fetch timelines: " + err.Error()})
		return
	}

	c.JSON(http.StatusOK, timelines)
}

/// this function either generates from existing story, xor from fragments
/// not both as the ordering would be much more complex to implement if both were supported
func CreateNewTimeline(db *gorm.DB, user *model.User, builder TimelineBuilder) (model.Timeline, error) {
	now := time.Now().Unix()
	var description = ""
	var timelineid = uuid.New()
	if builder.Description != nil {
		description = *builder.Description
	}

	tx := db.Begin()
	var timeline = model.Timeline {
		ID: timelineid.String(),
		Name: builder.Name,
		Description: description,
		Source: builder.Source.String(),
		Created: now,
		LastEdited: now,
	}

	tlerr := tx.Create(&timeline).Error
	if tlerr != nil {
		fmt.Printf("error creating timeline: %s\n", tlerr);
		tx.Rollback()
		return model.Timeline{}, tlerr
	}

	var relation = model.Relation {
		Parent: builder.Source.String(),
		Child: timelineid.String(),
		ParentCategory: builder.Category,
		ChildCategory: "timelines",
		Description: "",
	}

	_, result := CreateNewRelation(tx, &relation)
	if result != nil {
		tx.Rollback()
		return model.Timeline{}, result
	}

	gerr := CreateNewEntity(tx, timelineid, uuid.MustParse(user.DefaultGroup))
	if gerr != nil {
		tx.Rollback()
		return model.Timeline{}, gerr
	}

	fragments,serr := selectFragmentsByEntity(tx, builder.Source);
	if serr != nil {
		tx.Rollback()
		return model.Timeline{}, serr
	}

	for idx, fragment := range fragments {
		_, merr := CreateMoment(tx, timelineid, fragment.ID, int64(idx))
		if merr != nil {
			tx.Rollback()
			return model.Timeline{}, merr
		}
	}

	tx.Commit()
	return timeline, nil
}

func CreateTimeline(c *gin.Context) {
	user, uerr := auth.GetUserFromClaims(db.DB, c);

	if uerr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": uerr.Error()})
		return
	}
	fmt.Printf("in create timeline")
	var builder TimelineBuilder
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "invalid JSON submission"})
		return
	}

	jsonData, _ := json.Marshal(builder)
	fmt.Printf("received JSON: %s\n", jsonData)

	timeline, tlerr := CreateNewTimeline(db.DB, user, builder)

	if tlerr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create timeline"})
		return
	}

	full, err := GetFullTimeline(db.DB, timeline)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to get full timeline"})
	}
	c.JSON(http.StatusOK, full)
}