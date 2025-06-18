package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	
)

func RegisterLocationRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/locations/", auth.JWTMiddleware(), GetLocations)
	r.GET("/locations/:id", auth.JWTMiddleware(), GetLocation)
	r.POST("/locations/", auth.JWTMiddleware(), CreateLocation)
	r.GET("/locations/filter", auth.JWTMiddleware(), GetLocationsByStory)
	return r
}

type LocationBuilder struct {
	ID				uuid.UUID		`json:"id"`
	Name			string			`json:"name"`
	Description		*string			`json:"description"`
	Tags			[]model.Tag		`json:"tags"`
	Thumbnail		*model.Image	`json:"thumbnail"`
}

type LocationRender struct {
	ID				uuid.UUID	`json:"id"`
	Name			string	`json:"name"`
	Description		*string	`json:"description"`
	Thumbnail		*model.Image	`json:"thumbnail"`
}

func GetLocations(c *gin.Context) {
	// grab all stories where public = true
	var locations []model.Location
    err := db.DB.
        Where("entities.active = ?", true).
        Joins("JOIN entities ON entities.id = locations.id").
        Find(&locations).Error
	
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, locations)
}

func GetLocation(c *gin.Context) {
	idParam := c.Param("id")

	// Validate UUID
	locationID, err := uuid.Parse(idParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid UUID format"})
		return
	}

	// Query database
	var location model.Location
	if err := db.DB.First(&location, "id = ?", locationID).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "Location not found"})
		return
	}

	// Return result
	c.JSON(http.StatusOK, location)
}

func CreateNewLocation(builder LocationBuilder) (model.Location, error) {
	now := time.Now().Unix()

	var description = ""
	if builder.Description != nil {
		description = *builder.Description
	}

	var location = model.Location {
		ID: uuid.New().String(),
		Name: builder.Name,
		Description: description,
		Created: now,
	}

	err := db.DB.Create(&location).Error;
	return location, err
}

func CreateLocation(c *gin.Context) {
	var builder LocationBuilder
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid request: " + err.Error(),
		})
		return
	}

	location, dberr := CreateNewLocation(builder);
	if dberr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create location"})
		return
	}
	c.JSON(http.StatusOK, location)
}

func GetLocationsByStory(c *gin.Context) {
	IDString := c.Query("parent")
	parentID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse parent as UUID"})
		return
	}

	var locations []model.Location
	err := db.DB.
		Model(&model.Location{}).
		Joins("JOIN relations ON relations.child = locations.id").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, "stories", "locations").
		Find(&locations).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	locationRenders := make([]LocationRender, 0, len(locations))

	for _, loc := range locations {
		var thumbnail *model.Image
		if loc.Thumbnail != "" {
			var img model.Image
			if err := db.DB.First(&img, "id = ?", loc.Thumbnail).Error; err == nil {
				thumbnail = &img
			} else {
				fmt.Printf("warning: failed to load thumbnail for location %s: %v\n", loc.ID, err)
			}
		}

		render := LocationRender{
			ID:          uuid.MustParse(loc.ID),
			Name:        loc.Name,
			Description: &loc.Description,
			Thumbnail:   thumbnail,
		}
		locationRenders = append(locationRenders, render)
	}

	c.JSON(http.StatusOK, locationRenders)
}