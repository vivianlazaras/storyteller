function delete_entity(id) {
    if (!confirm("Are you sure you want to delete this item? This action cannot be undone.")) {
        return; // User cancelled
    }
    
    fetch('/stories/' + id, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json'
        }
    }).then(response => {
        if (response.redirected) {
            window.location.href = response.url;
        }
        else if (response.ok) {
            // handle success
            
        } else {
            // handle error
            alert('Delete failed.');
        }
    });
}