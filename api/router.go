package main

import (
    "github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/handlers"
)

func SetupRouter() *gin.Engine {
    r := gin.Default()

    handlers.RegisterUserRoutes(r)
    handlers.RegisterStoryRoutes(r)
    // Repeat for other models

    return r
}