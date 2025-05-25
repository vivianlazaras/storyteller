package main

import (
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"fmt"
	"log"
)

func main() {
	var path = "../config.json"
	config, err := LoadConfig(path)
	if err != nil {
		fmt.Printf("failed to laod config exiting");
		return
	}

	var realmURL = config.Server.Oidc.IssuerUrl
	// Secure routes
	/*
	secured.Use(oidc.RequireAuth())
	*/
	if err := auth.InitJWKS(realmURL); err != nil {
		log.Fatalf("Failed to load JWKS: %v", err)
	}

	addr := fmt.Sprintf("%s:%d", config.Api.Server.Listen, config.Api.Server.Port)
	fmt.Printf("address: %s", addr)
	db.InitDB();
    r, err := SetupRouter(&config)
	if err != nil {
		return
	}
    r.Run(addr)
}