package main

import (
    "github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/handlers"
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

func SetupRouter(config *Config) (*gin.Engine, error) {
    r := gin.Default()


    handlers.RegisterUserRoutes(r)
    handlers.RegisterStoryRoutes(r)
    // Repeat for other models

    return r, nil
}