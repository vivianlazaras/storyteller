package db

import (
	"log"
	"gorm.io/gorm"
	"gorm.io/driver/postgres"
	"gorm.io/gen"
)

var DB *gorm.DB

func InitDB() *gorm.DB {
    dsn := "host=localhost user=storyteller password=password dbname=storyteller port=5432 sslmode=disable"
    database, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})
    if err != nil {
        log.Fatalf("Failed to connect to database: %v", err)
    }

	g := gen.NewGenerator(gen.Config{
        OutPath: "./model", // where generated models & query code go
        Mode: gen.WithoutContext,
        FieldNullable: true, // map nullable columns to pointer types
    })

	g.WithDataTypeMap(map[string]func(gorm.ColumnType) string{
        "uuid": func(columnType gorm.ColumnType) string {
            return "uuid.UUID"
        },
    })
    g.WithImportPkgPath("github.com/google/uuid")

    g.UseDB(database)
    g.GenerateAllTable()
    g.Execute()

    DB = database
    return DB
}