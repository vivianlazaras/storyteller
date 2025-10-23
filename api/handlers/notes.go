package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

func RegisterNoteRoutes(r *gin.Engine) *gin.Engine {
	r.POST("/notes/", auth.JWTMiddleware(), CreateNote)
	r.PUT("/notes/complete/:id", auth.JWTMiddleware(), CompleteNote)
	r.PUT("/notes/:id", auth.JWTMiddleware(), EditNote)
	r.GET("/notes/", auth.JWTMiddleware(), ListNotes)
	r.DELETE("/notes/:id", auth.JWTMiddleware(), DeleteNote)
	return r
} 

type NoteBuilder struct {
	Name		string		`json:"name"`
	Description *string		`json:"description"`
	Deadline	*int64		`json:"deadline"`
	Parent		uuid.UUID 	`json:"parent"`
	Category	string		`json:"category"`
}

func CreateNewNote(tx *gorm.DB, builder NoteBuilder, userID, groupID uuid.UUID) (model.Note, error) {
	now := time.Now().Unix()


	newnote := model.Note {
		ID: uuid.New(),
		Name: builder.Name,
		Description: builder.Description,
		Created: now,
	}

	// now that note should be linked through the Relations table
	var relation = model.Relation {
		Parent: builder.Parent,
		ParentCategory: builder.Category,
		Child: newnote.ID,
		ChildCategory: "notes",
	}

	err := tx.Create(&newnote).Error
	if err != nil {
		return model.Note{}, err
	}

	// this will also check to ensure the user has access to the group, so that logic is in one place
	if err := CreateNewEntity(tx, newnote.ID, userID, groupID); err != nil {
		return model.Note{}, err
	}

	_, relresult := CreateNewRelation(tx, &relation)
	if relresult != nil {
		return model.Note{}, relresult
	}

	return newnote, nil
}

func CreateNote(c *gin.Context) {

	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized: " + err.Error()})
		return
	}
	// I should really add an assertion of category here, but it's fine for now
	
	var builder NoteBuilder
	if jsonErr := c.ShouldBindJSON(&builder); jsonErr != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse note creation json"})
	}

	// Begin transaction
	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create transaction"})
		return
	}

	note, err := CreateNewNote(tx, builder, user.ID, user.DefaultGroup)
	if err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "inernal server error creating note"})
	}

	if err := tx.Model(&model.Entity{}).
		Where("id = ?", note.ID).
		Update("group_id", user.DefaultGroup).Error; err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update group_id: " + err.Error()})
		return
	}

	// Commit transaction
	if err := tx.Commit().Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to commit transaction"})
		return
	}

	c.JSON(http.StatusOK, note)
}

func CompleteNote(c *gin.Context) {
	noteID, result := GetIDParam(c.Param("id"))
	if result.IsError() {
		result.GinResult(c)
		return
	}

	now := time.Now().Unix()

	// Update the note's `completed` field
	if err := db.DB.Model(&model.Note{}).Where("id = ?", noteID).Update("completed", now).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to complete note"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "note marked as completed", "completed_at": now})
}
func EditNote(c *gin.Context) {
	/*noteID, result := GetIDParam(c.Param("id"))
	if result.IsError() {
		result.GinResult(c)
		return
	}*/


}

/// list notes by entity id
func ListNotes(c *gin.Context) {
	entityID, result := GetIDParam(c.Query("parent"))
	if result.IsError() {
		result.GinResult(c)
		return
	}

	var notes []model.Note
	err := db.DB.
		Model(&model.Note{}).
		Joins("JOIN relations ON relations.child = notes.id").
		Where("relations.parent = ? AND relations.child_category = ?", entityID, "notes").
		Find(&notes).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to retrieve notes"})
		return
	}

	c.JSON(http.StatusOK, notes)
}

func DeleteNote(c *gin.Context) {}