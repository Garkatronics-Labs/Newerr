#+feature dynamic-literals

package errors

// general
Err_Test :: Error_Id(0001)

error_catalog := map[Error_Id]Error_Meta{
    Test = {
        message  = "",
        hint     = "",
        severity = "warning",
        category = "general",
    },
}
