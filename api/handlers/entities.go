package handlers

import (
    "net/http"

    "github.com/gin-gonic/gin"
    "github.com/vivianlazaras/storyteller/db"
    "github.com/vivianlazaras/storyteller/model"
)

func RegisterEntityRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/relations", ListEntitiesByChildCategory)
	r.POST("/relations/", CreateRelation)
	return r
}

type RelatedEntity struct {
    ID   string `json:"id"`
    Name string `json:"name"`
}

func ListEntitiesByChildCategory(c *gin.Context) {
    childCategory := c.Query("category")
    if childCategory == "" {
        c.JSON(http.StatusBadRequest, gin.H{"error": "child_category query parameter is required"})
        return
    }

    var entities []RelatedEntity
    var query string

    switch childCategory {
    case "characters":
        query = `SELECT id, name FROM characters`
    case "stories":
        query = `SELECT id, name FROM stories`
    case "fragments":
        query = `SELECT id, name FROM fragments`
    case "locations":
        query = `SELECT id, name FROM locations`
    default:
        c.JSON(http.StatusBadRequest, gin.H{"error": "unknown child_category"})
        return
    }

    err := db.DB.Raw(query).Scan(&entities).Error
    if err != nil {
        c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
        return
    }

    c.JSON(http.StatusOK, entities)
}

func CreateRelation(c *gin.Context) {
	var relation model.Relation

	if err := c.ShouldBindJSON(&relation); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request: " + err.Error()})
		return
	}

	// Optionally: check if relation already exists
	exists := false
	err := db.DB.
		Table("relations").
		Where("parent = ? AND child = ? AND parent_category = ? AND child_category = ?",
			relation.Parent, relation.Child, relation.ParentCategory, relation.ChildCategory).
		Select("count(*) > 0").
		Find(&exists).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to check existing relation"})
		return
	}
	if exists {
		c.JSON(http.StatusConflict, gin.H{"error": "Relation already exists"})
		return
	}

	if err := db.DB.Table("relations").Create(&relation).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create relation: " + err.Error()})
		return
	}

	c.JSON(http.StatusOK, relation)
}