#+feature dynamic-literals

package errors

// Returns true if `id` lies inside the open (start, end) category range.
err_in_cat :: proc(id: Error_Id, start: Error_Id, end: Error_Id) -> bool {
	return id > start && id < end
}

// --- error declarations per category ---

// general — errors [0001 .. 0001]
Err_Cat_general_Start :: Error_Id(0001)
Err_Cat_general_End   :: Error_Id(0001)
Err_Test :: Error_Id(0001)


// --- error metadata catalog ---
error_catalog := map[Error_Id]Error_Meta{
    Err_Test = {
        message  = "",
        hint     = "",
        severity = "warning",
        category = "general",
    },
}
