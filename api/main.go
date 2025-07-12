package main

import (
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/config"
	"log"
	"fmt"
)

func main() {
	var path = "../config.json"
	config, err := config.LoadConfig(path)
	if err != nil {
		fmt.Printf("failed to laod config exiting");
		return
	}

	//var realmURL = config.Server.Oidc.IssuerUrl
	if config.Api.Server.SelfHostedAuth {
		if config.Api.Server.KeyFile == nil {
			fmt.Printf("Private KeyFile required for self hosted authentication")
			return
		}
		var autherr = auth.InitAuth(config.Api.Server.Oidc, *config.Api.Server.KeyFile)
		if autherr != nil {
			fmt.Printf("failed loading private keyfile: " + autherr.Error())
			return
		}

		log.Println("INFO: successfully loaded JWT key")
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