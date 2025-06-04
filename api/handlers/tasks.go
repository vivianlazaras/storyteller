package handlers

import (
	"net/http"
	"time"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/google/uuid"
)

func RegisterTaskRoutes(r *gin.Engine) *gin.Engine {
	r.POST("/tasks/", CreateTask)
	r.PUT("/tasks/complete/:id", CompleteTask)
	r.PUT("/tasks/:id", EditTask)
	r.GET("/tasks", ListTasks)
	r.DELETE("/tasks/:id", DeleteTask)
	return r
} 

type CreateTaskParts struct {
	Name		string	`json:"name"`
	Description *string	`json:"description"`
	Deadline	*int64	`json:"deadline"`
}

func CreateTaskFromParts(task CreateTaskParts) (model.Task, error) {
	now := time.Now().Unix()
	description := ""

	if task.Description != nil {
		description = *task.Description
	}

	newtask := model.Task {
		ID: uuid.New().String(),
		Name: task.Name,
		Description: description,
		Created: now,
	}

	err := db.DB.Create(&newtask).Error
	return newtask, err
}

func CreateTask(c *gin.Context) {
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
	
	var parts CreateTaskParts
	if jsonErr := c.ShouldBindJSON(&parts); jsonErr != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse task creation json"})
	}

	task, err := CreateTaskFromParts(parts)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "inernal server error creating task"})
	}

	// now that task should be linked through the Relations table
	var relation = model.Relation {
		Parent: entity.String(),
		ParentCategory: category,
		Child: task.ID,
		ChildCategory: "tasks",
	}

	relresult := CreateNewRelation(&relation)
	if !relresult.IsError() {
		c.JSON(http.StatusOK, task)
	}
}

func CompleteTask(c *gin.Context) {
	taskID, result := GetIDParam(c.Param("id"))
	if result.IsError() {
		result.GinResult(c)
		return
	}

	now := time.Now().Unix()

	// Update the task's `completed` field
	if err := db.DB.Model(&model.Task{}).Where("id = ?", taskID).Update("completed", now).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to complete task"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "task marked as completed", "completed_at": now})
}
func EditTask(c *gin.Context) {
	/*taskID, result := GetIDParam(c.Param("id"))
	if result.IsError() {
		result.GinResult(c)
		return
	}*/


}

/// list tasks by entity id
func ListTasks(c *gin.Context) {
	entityID, result := GetIDParam(c.Query("entity"))
	if result.IsError() {
		result.GinResult(c)
		return
	}

	var tasks []model.Task
	err := db.DB.
		Model(&model.Task{}).
		Joins("JOIN relations ON relations.child = tasks.id").
		Where("relations.parent = ? AND relations.child_category = ?", entityID, "tasks").
		Find(&tasks).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to retrieve tasks"})
		return
	}

	c.JSON(http.StatusOK, tasks)
}

func DeleteTask(c *gin.Context) {}