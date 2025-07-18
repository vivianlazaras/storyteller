package db

import (
	"log"
	"gorm.io/gorm"
	"gorm.io/driver/postgres"
	"net/http"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

var DB *gorm.DB

func InitDB() *gorm.DB {
    dsn := "host=localhost user=storyteller password=password dbname=storyteller port=5432 sslmode=disable"
    database, err := gorm.Open(postgres.Open(dsn), &gorm.Config{})
    if err != nil {
        log.Fatalf("Failed to connect to database: %v", err)
    }

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

func GetByID[T any](tableName, id string) (*T, error) {

	var result = new(T)
	if err := DB.Table(tableName).First(&result, "id = ?", id).Error; err != nil {
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

	result, err := GetByID[T](tableName, id.String())
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return nil, err
	}

	return result, nil
}