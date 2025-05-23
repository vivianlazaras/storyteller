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
	addr := fmt.Sprintf("%s:%d", config.Api.Server.Listen, config.Api.Server.Port)
	fmt.Printf("address: %s", addr)
	db.InitDB();
    r := SetupRouter()
    r.Run(addr)
}