package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/middleware"
	"github.com/vivianlazaras/storyteller/db"
)

func RegisterUserRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/users", ListUsers)
    r.GET("/users/:id", GetUser)
    r.POST("/users", middleware.RequireOIDC(), CreateUser)
    r.PUT("/users/:id", middleware.RequireOIDC(), UpdateUser)
    r.DELETE("/users/:id", middleware.RequireOIDC(), DeleteUser)
	return r
}

func ListUsers(c *gin.Context) {
	// Replace with actual DB query
	c.JSON(http.StatusOK, []model.User{})
}

func GetUser(c *gin.Context) {
	db.GetByCtxID[model.User](c, "users")
}

func CreateUser(c *gin.Context) {
	c.JSON(http.StatusOK, []model.User{})
}

func UpdateUser(c *gin.Context) {
	var user model.User
	if err := c.ShouldBindJSON(&user); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	user.ID = c.Param("id")
	c.JSON(http.StatusOK, user)
}

func DeleteUser(c *gin.Context) {
	id := c.Param("id")
	c.JSON(http.StatusOK, gin.H{"deleted": id})
}


func getUserByEmail(email string) (model.User, error) {
	var result = new(model.User);
	if err := db.DB.Table("users").First(&result, "email = ?", id).Error; err != nil {
		return nil, err
	}

	return result, nil
}