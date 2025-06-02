package handlers

import (
	"net/http"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterLocationRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/locations/", GetLocations)
	r.GET("/locations/:id", GetLocation)
	return r
}

func GetLocations(c *gin.Context) {
	// grab all stories where public = true
	var locations []model.Location
    err := db.DB.
        Where("metadata.public = ?", true).
        Joins("JOIN metadata ON metadata.id = locations.metadata").
        Find(&locations).Error
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, locations)
}

func GetLocation(c *gin.Context) {

}