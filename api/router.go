package main

import (
    "github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/handlers"
    "github.com/vivianlazaras/storyteller/auth"
    "github.com/vivianlazaras/storyteller/config"
    "fmt"
    "io/ioutil"
)

func ReadFileAsString(path string) (string, error) {
	data, err := ioutil.ReadFile(path) // use os.ReadFile in Go 1.16+
	if err != nil {
		return "", fmt.Errorf("failed to read file %s: %w", path, err)
	}
	return string(data), nil
}

func SetupRouter(config *config.Config) (*gin.Engine, error) {
    r := gin.Default()

    handlers.RegisterStoryRoutes(r)
    handlers.RegisterAnalyticsRoutes(r)
    handlers.RegisterCharacterRoutes(r)
    handlers.RegisterLocationRoutes(r)
    handlers.RegisterTagRoutes(r)
    handlers.RegisterTimelineRoutes(r)
    handlers.RegisterFragmentRoutes(r)
    handlers.RegisterEntityRoutes(r)
    handlers.RegisterNoteRoutes(r)
    handlers.RegisterImageRoutes(r)
    handlers.RegisterProfileRoutes(r)
    handlers.RegisterGroupRoutes(r)
    auth.RegisterAuthRoutes(r)
    // Repeat for other models

    return r, nil
}