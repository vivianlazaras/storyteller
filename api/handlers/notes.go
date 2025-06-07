package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/google/uuid"
)

func RegisterNoteRoutes(r *gin.Engine) *gin.Engine {
	r.POST("/notes/", CreateNote)
	r.PUT("/notes/complete/:id", CompleteNote)
	r.PUT("/notes/:id", EditNote)
	r.GET("/notes/", ListNotes)
	r.DELETE("/notes/:id", DeleteNote)
	return r
} 

type CreateNoteParts struct {
	Name		string	`json:"name"`
	Description *string	`json:"description"`
	Deadline	*int64	`json:"deadline"`
}

func CreateNoteFromParts(note CreateNoteParts) (model.Note, error) {
	now := time.Now().Unix()
	description := ""

	if note.Description != nil {
		description = *note.Description
	}

	newnote := model.Note {
		ID: uuid.New().String(),
		Name: note.Name,
		Description: description,
		Created: now,
	}

	err := db.DB.Create(&newnote).Error
	return newnote, err
}

func CreateNote(c *gin.Context) {
	category := c.Query("category")

	// ensure parameters are properly set
	if category == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "missing required query parameter"})
	}

	entity, result := GetIDParam(c.Query("entity"))
	if result.IsError() {
		result.GinResult(c)
		return
	}

	// I should really add an assertion of category here, but it's fine for now
	
	var parts CreateNoteParts
	if jsonErr := c.ShouldBindJSON(&parts); jsonErr != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse note creation json"})
	}

	note, err := CreateNoteFromParts(parts)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "inernal server error creating note"})
	}

	// now that note should be linked through the Relations table
	var relation = model.Relation {
		Parent: entity.String(),
		ParentCategory: category,
		Child: note.ID,
		ChildCategory: "notes",
	}

	relresult := CreateNewRelation(&relation)
	if !relresult.IsError() {
		c.JSON(http.StatusOK, note)
	}
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