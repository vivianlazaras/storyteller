package handlers

import (
	"regexp"
	"net/http"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/auth"
	"gorm.io/gorm"
	"fmt"
	"encoding/json"
)

type Result struct {
	Status		int
	Message 	string
}

func (result *Result) GinResult(c *gin.Context) {
	c.JSON(result.Status, gin.H{"error": result.Message})
}

func NewResult(status int, message string) Result {
	return Result {
		Status: status,
		Message: message,
	}
}

func (result *Result) IsError() bool {
	return result.Status != http.StatusOK
}

// RemoveJavaScriptTags removes all <script>...</script> blocks from a Markdown string.
/// This is nessisary to prevent javascript injections that could potentially be used to
/// gain access to other peoples systems.
func RemoveJavaScriptTags(markdown string) string {
	// (?i) makes it case-insensitive, (?s) allows . to match newlines
	re := regexp.MustCompile(`(?is)<script.*?>.*?</script>`)
	return re.ReplaceAllString(markdown, "")
}

func GetIDParam(idtext string) (*uuid.UUID, Result) {

	if idtext == "" {
		return nil, NewResult(http.StatusBadRequest, "missing task id")
	}
	parsedID, parseErr := uuid.Parse(idtext);
	if parseErr != nil {
		return nil, NewResult(http.StatusBadRequest, "improperly formatted UUID")
	}

	return &parsedID, NewResult(http.StatusOK, "successfully parsed ID")
}

func strPtr(value string) *string {
	return &value
}

func GetID(c *gin.Context) (uuid.UUID, error) {
	idParam := c.Param("id")
	id, err := uuid.Parse(idParam)
	if err != nil {
		return uuid.New(), fmt.Errorf("invalid UUID: %w", err)
	}

	return id, nil
}

func GetByID[T any](db *gorm.DB, tableName string, id, userID uuid.UUID) (*T, error) {

	access, _, permerr := CheckUserEntityPermission(db, userID, id, "read")
	if permerr != nil {
		fmt.Println("unknown perm error")
		return nil, permerr
	}

	if access != true {
		fmt.Println("access to entity denied")
		return nil, fmt.Errorf("access to entity denied")
	}

	var result = new(T)
	if err := db.Table(tableName).First(&result, "id = ?", id).Error; err != nil {
		return nil, err
	}

	return result, nil
	// c.JSON(http.StatusOK, result)
}

func GetByCtxID[T any](db *gorm.DB, c *gin.Context, tableName string) (*T, error) {
	user, uerr := auth.GetUserFromClaims(db, c)
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": uerr})
		return nil, uerr
	}
	id, err := GetID(c)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return nil, err
	}

	result, err := GetByID[T](db, tableName, id, user.ID)
	if err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": err.Error()})
		return nil, err
	}

	return result, nil
}

/// a convience for when there's a mismatch of JSON fields between the API and the client implementation.
func PrintObjDebug(obj any) {
    jsonBytes, err := json.MarshalIndent(obj, "", "  ") // Use json.Marshal for compact output
    if err != nil {
        fmt.Println("Error marshalling to JSON:", err)
        return
    }

    fmt.Println(string(jsonBytes))
}