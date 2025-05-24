package main

import (
    "github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/handlers"
	"github.com/vivianlazaras/storyteller/middleware"
    "fmt"
    "io/ioutil"
    "log"
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

    var SecretFile = config.Api.Server.Oidc.SecretFile
    clientSecret, err := ReadFileAsString(SecretFile)
    if err != nil {
		fmt.Printf("failed to get client secret exiting: %s", err);
		return nil, err
	}

	oidcConfig := middleware.Config{
		Issuer:       config.Api.Server.Oidc.IssuerUrl,
		ClientID:     config.Api.Server.Oidc.ClientID,
		ClientSecret: clientSecret,
	}

	oidc, err := middleware.New(&oidcConfig)
	if err != nil {
		log.Fatalf("OIDC setup failed: %v", err)
	}

    handlers.RegisterUserRoutes(r)
    handlers.RegisterStoryRoutes(r, oidc)
    // Repeat for other models

    return r, nil
}