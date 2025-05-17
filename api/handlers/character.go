package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/middleware"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterUser(r *gin.Engine) *gin.Engine {
	r.GET("/characters", ListCharacters)
    r.GET("/characters/:id", GetCharacter)
    r.POST("/characters", middleware.RequireOIDC(), CreateCharacter)
    r.PUT("/characters/:id", middleware.RequireOIDC(), UpdateCharacter)
    r.DELETE("/characters/:id", middleware.RequireOIDC(), DeleteCharacter)
	return r
}

// for now this route can only fetch public characters
func ListCharacters(c *gin.Context) {
	// grab all stories where public = true
	c.JSON(http.StatusOK, []model.Story{})
}

func GetCharacter(c *gin.Context) {
	character, err := db.GetByCtxID[model.Character](c, "characters")
	if err != nil {
		return
	}

	metadata, err := db.GetByID[model.Metadatum]("metadata", character.Metadata)
	if err != nil || metadata.Public != true {
		c.JSON(http.StatusNotFound, model.Character{})
		return
	}

	c.JSON(http.StatusOK, character)
}

func CreateCharacter(c *gin.Context) {
	c.JSON(http.StatusOK, model.Character{})
}

func UpdateCharacter(c *gin.Context) {
	c.JSON(http.StatusOK, model.Character{})
}

func DeleteCharacter(c *gin.Context) {
	c.JSON(http.StatusOK, model.Character{})
}
