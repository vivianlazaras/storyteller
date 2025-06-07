package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	
)

func RegisterLocationRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/locations/", GetLocations)
	r.GET("/locations/:id", GetLocation)
	r.POST("/locations/", CreateLocation)
	return r
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

type LocationBuilder struct {
	Name			string	`json:"name"`
	Description		*string	`json:"description"`
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