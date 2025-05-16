package main

import (
	"github.com/vivianlazaras/storyteller/db"
)

func main() {
	db.InitDB();
    r := SetupRouter()
    r.Run() // default :8080
}