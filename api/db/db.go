package db

import (
	"log"
	"gorm.io/gorm"
	"gorm.io/driver/postgres"
	"net/http"
	"fmt"
	"github.com/gin-gonic/gin"
	"gorm.io/gen"
	"github.com/google/uuid"
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

func GetID(c *gin.Context) (*uuid.UUID, error) {
	idParam := c.Param("id")
	userID, err := uuid.Parse(idParam)
	if err != nil {
		return nil, fmt.Errorf("invalid UUID: %w", err)
	}

	if DB == nil {
		return nil, fmt.Errorf("database is uninitialized")
	}

	return &userID, nil
}

func GetByID[T any](tableName string, id *uuid.UUID) (*T, error) {

	var parsedID uuid.UUID
	if id == nil {
		return nil, fmt.Errorf("missing ID in call to GetByID");
	}else{
		parsedID = *id
	}

	var result = new(T)
	if err := DB.Table(tableName).First(&result, "id = ?", parsedID).Error; err != nil {
		return nil, err
	}

	return result, nil
	// c.JSON(http.StatusOK, result)
}

func GetByCtxID[T any](c *gin.Context, tableName string) (*T, error) {
	id, err := GetID(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return nil, err
	}

	result, err := GetByID[T](tableName, id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return nil, err
	}

	return result, nil
}