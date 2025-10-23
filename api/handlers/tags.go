package handlers

import (
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/google/uuid"
	"github.com/gin-gonic/gin"
    "net/http"
	"gorm.io/gorm"
)

func RegisterTagRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/tags/:id", GetTagsByEntityID)
	return r
}

func InsertTagsForEntity(tx *gorm.DB, entityID uuid.UUID, tags []string) error {
	for _, tag := range tags {
		var newtag = model.Tag {
			ID: uuid.New(),
			Value: tag,
			Entity: entityID,
		}
		err := tx.Create(&newtag).Error
		if err != nil {
			return err
		}
	}
	return nil
}

func SelectTagsByEntityID(entityID uuid.UUID) ([]model.Tag, error) {

    var tags []model.Tag
    if err := db.DB.Where("entity = ?", entityID).Find(&tags).Error; err != nil {
        return []model.Tag{}, err
    }

    return tags, nil
}

func GetTagsByEntityID(c *gin.Context) {
	entityIDStr := c.Param("id")
    
    entityID, err := uuid.Parse(entityIDStr)
	if err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid entity ID"})
        return
    }

	tags, tagerr := SelectTagsByEntityID(entityID);
	if tagerr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error: ": tagerr})
		return
	} 
	c.JSON(http.StatusOK, tags)
}