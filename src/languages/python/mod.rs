mod current_context;
mod current_line;
mod current_scope;
mod function_parameters;
mod function_summary;
mod syntax;

pub use current_context::describe_current_context;
pub use current_line::describe_current_line;
pub use current_scope::describe_current_scope;
pub use function_parameters::describe_function_parameter_list;
pub use function_summary::describe_function_summary;