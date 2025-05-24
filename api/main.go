package main

import (
	"github.com/vivianlazaras/storyteller/db"
	"fmt"
)

func main() {
	var path = "../config.json"
	config, err := LoadConfig(path)
	if err != nil {
		fmt.Printf("failed to laod config exiting");
		return
	}
	// Secure routes
	/*
	secured.Use(oidc.RequireAuth())
	*/

	addr := fmt.Sprintf("%s:%d", config.Api.Server.Listen, config.Api.Server.Port)
	fmt.Printf("address: %s", addr)
	db.InitDB();
    r, err := SetupRouter(&config)
	if err != nil {
		return
	}
    r.Run(addr)
}