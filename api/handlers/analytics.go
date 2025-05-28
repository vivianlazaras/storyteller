package handlers

import (
	"net/http"
	"strconv"
	"github.com/gin-gonic/gin"
	
	"github.com/vivianlazaras/storyteller/db"
)

type TagCount struct {
    Value string `json:"value"`
    Count int    `json:"count"`
}

func RegisterAnalyticsRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/analytics/populartags", GetTopTags)
	return r
}

func GetTopTags(c *gin.Context) {
    
	var results []TagCount

	var minCount = 0
	var limit = 10

	if mc := c.Query("min_count"); mc != "" {
		if parsed, err := strconv.Atoi(mc); err == nil {
			minCount = parsed
		} else {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid min_count value"})
			return
		}
	}

	if lim := c.Query("limit"); lim !=  "" {
		if parsed, err := strconv.Atoi(lim); err == nil {
			limit = parsed
		} else {
			c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid limit value"})
			return
		}
	}

	if limit > 100 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid limit value, maximum value 100"})
		return
	}

	err := db.DB.
		Table("tags").
		Select("value, COUNT(*) as count").
		Group("value").
		Having("COUNT(*) >= ?", minCount).
		Order("count DESC").
		Limit(limit).
		Scan(&results).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to query tags"})
		return
	}

	c.JSON(http.StatusOK, results)
    
}