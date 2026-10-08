/// Create a function for calling a single endpoint
/// with a GET request.
///
/// # Example
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_get;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_get!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Public,
///         Vec<u64>,
///     );
///
/// }
/// # fn main() {}
/// ```
///
/// ## Result:
///
/// ```rust,ignore
/// /// Docs for the generated function
/// pub async fn function_name(&self) -> EsiResult<Vec<u64>> {
///     let path = self.esi.get_endpoint_for_op_id("some_operation_id")?;
///     self.esi
///         .query("GET", RequestType::Public, &path, None, None)
///         .await
/// }
/// ```
///
/// Additionally, this macro supports path replacements to insert variables
/// into the path from ESI.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_get;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_get!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Public,
///         Vec<u64>,
///         (alliance_id: i64) => "{alliance_id}"
///     );
///
/// }
/// # fn main() {}
/// ```
/// ## Result:
///
/// ```rust,ignore
/// /// Docs for the generated function
/// pub async fn function_name(&self, alliance_id: i64) -> EsiResult<Vec<u64>> {
///     let path = self.esi.get_endpoint_for_op_id("some_operation_id")?
///         .replace("{alliance_id}", &alliance_id.to_string());
///     self.esi
///         .query("GET", RequestType::Public, &path, None, None)
///         .await
/// }
/// ```
///
/// Finally, there is support for required and optional query params. These are different from path
/// parameters: in 'markets/{region_id}/orders?page=1', region_id is a path parameter and page is a
/// query parameter. Note that in the macro invocation, query parameters are separated from path
/// parameters with a semicolon, and that optional query parameters always follow required ones.
/// See [crate::groups::MarketGroup] for sample macro calls.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_get;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_get!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Public,
///         Vec<u64>,
///         (region_id: u64) => "{region_id}";
///         (page: i32) => "page";
///         Optional(order_type: bool) => "order_type"
///     );
///
/// }
/// # fn main() {}
/// ```
/// ## Result:
///
/// ```rust,ignore
/// /// Docs for the generated function
/// pub async fn function_name(&self, region_id: u64, page: i32, order_type: Option<bool>) -> EsiResult<Vec<u64>> {
///     let path = self.esi.get_endpoint_for_op_id("some_operation_id")?
///         .replace("{region_id}", &region_id.to_string());
///     let params = vec![
///         ("page", page.to_string()),
///     ]
///     let mut params = params;
///     if let Some(order_type) = order_type {
///         params.push(("order_type", order_type.to_string()))
///     }
///     let params: Vec<(&str, &str)> = params.iter().map(|(a, b)| (*a, &**b)).collect();
///     self.esi
///         .query("GET", RequestType::Public, &path, Some(&params), None)
///         .await
/// }
/// ```
///
/// # Extended query parameters
///
/// `api_get!`, `api_post!`, `api_put!` and `api_delete!` also accept query
/// parameters tagged with a kind, after a `;` that follows the path parameters:
///
/// - `Required(name: T)`: a required value, sent as `name=value`
/// - `Optional(name: T)`: the function takes an `Option<T>`
/// - `Many(name: &[T])`: a required list, sent as repeated keys (`name=1&name=2`)
/// - `OptionalMany(name: &[T])`: the function takes an `Option<&[T]>`
///
/// `api_post!` and `api_put!` take the body after a second `;`.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::{api_get, api_post};
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_get!(
///         /// Docs for the generated function
///         list_things,
///         "some_operation_id",
///         RequestType::Authenticated,
///         Vec<u64>,
///         (character_id: i64) => "{character_id}";
///         OptionalMany(labels: &[i64]) => "labels",
///         Optional(last_id: i64) => "last_id"
///     );
///
///     api_post!(
///         /// Docs for the generated function
///         add_things,
///         "some_other_operation_id",
///         RequestType::Authenticated,
///         Vec<i64>,
///         (character_id: i64) => "{character_id}";
///         Required(standing: f64) => "standing",
///         Optional(watched: bool) => "watched";
///         ids: &[i64]
///     );
///
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! api_get {
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
    ) => {
        $(#[$m])*
        pub async fn $fn_name(&self, $( $param: $param_t, )*) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            self.esi.
                query("GET", $visibility, &path, None, None)
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        $( ; $( ($qparam:ident: $qparam_t:ty) => $qreplace:literal ),+ )?
        $( ; $( Optional($opt_qparam:ident: $opt_qparam_t:ty) => $opt_qreplace:literal ),+ )?
    ) => {
        $(#[$m])*
        pub async fn $fn_name(
            &self,
            $( $param: $param_t, )*
            $($( $qparam: $qparam_t, )*)?
            $($( $opt_qparam: Option<$opt_qparam_t>, )*)?
        ) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            let params = vec![
                $($(
                    ($qreplace, $qparam.to_string()),
                )+)?
            ];
            $(
                let mut params = params; // avoids unnecessary 'mut' warning
                $(
                    if let Some($opt_qparam) = $opt_qparam {
                        params.push(($opt_qreplace, $opt_qparam.to_string()));
                    }
                )+
            )?
            let params: Vec<(&str, &str)> = params.iter().map(|(a, b)| (*a, &**b)).collect();
            self.esi.
                query("GET", $visibility, &path, Some(&params), None)
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; $( $qkind:ident($qparam:ident: $qparam_t:ty) => $qkey:literal ),+ $(,)?
    ) => {
        $crate::__esi_endpoint!(
            $(#[$m])*
            $fn_name, $op_id, $visibility, $ret_type, "GET",
            [ $( ($param: $param_t) => $replace ),* ]
            [ $( $qkind($qparam: $qparam_t) => $qkey ),+ ]
            [ ]
        );
    };
}

/// Internal: the type of a query parameter, given its kind.
#[doc(hidden)]
#[macro_export]
macro_rules! __esi_query_type {
    (Required, $t:ty) => { $t };
    (Optional, $t:ty) => { Option<$t> };
    (Many, $t:ty) => { $t };
    (OptionalMany, $t:ty) => { Option<$t> };
}

/// Internal: add a query parameter to the list of `(key, value)` pairs.
/// List parameters are sent as repeated keys (`labels=1&labels=2`).
#[doc(hidden)]
#[macro_export]
macro_rules! __esi_push_query {
    ($params:ident, Required, $name:ident, $key:literal) => {
        $params.push(($key, $name.to_string()));
    };
    ($params:ident, Optional, $name:ident, $key:literal) => {
        if let Some(value) = $name {
            $params.push(($key, value.to_string()));
        }
    };
    ($params:ident, Many, $name:ident, $key:literal) => {
        for value in $name.iter() {
            $params.push(($key, value.to_string()));
        }
    };
    ($params:ident, OptionalMany, $name:ident, $key:literal) => {
        if let Some(list) = $name {
            for value in list.iter() {
                $params.push(($key, value.to_string()));
            }
        }
    };
}

/// Internal: serialize the request body, if the endpoint has one.
#[doc(hidden)]
#[macro_export]
macro_rules! __esi_body {
    () => {
        None::<String>
    };
    ($body:ident) => {
        Some(serde_json::to_string($body)?)
    };
}

/// Internal: builds an endpoint function with path parameters, typed query
/// parameters (`Required`, `Optional`, `Many` or `OptionalMany`) and an
/// optional body. Used by the extended forms of `api_get!`, `api_post!`,
/// `api_put!` and `api_delete!`.
#[doc(hidden)]
#[macro_export]
macro_rules! __esi_endpoint {
    (
        $(#[$m:meta])*
        $fn_name:ident, $op_id:literal, $visibility:expr, $ret_type:ty, $method:literal,
        [ $( ($param:ident: $param_t:ty) => $replace:literal ),* ]
        [ $( $qkind:ident($qparam:ident: $qparam_t:ty) => $qkey:literal ),* ]
        [ $( $body_param:ident: $body_t:ty )? ]
    ) => {
        $(#[$m])*
        #[allow(clippy::vec_init_then_push)]
        pub async fn $fn_name(
            &self,
            $( $param: $param_t, )*
            $( $qparam: $crate::__esi_query_type!($qkind, $qparam_t), )*
            $( $body_param: $body_t, )?
        ) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            #[allow(unused_mut)]
            let mut params: Vec<(&str, String)> = Vec::new();
            $(
                $crate::__esi_push_query!(params, $qkind, $qparam, $qkey);
            )*
            let params: Vec<(&str, &str)> = params.iter().map(|(a, b)| (*a, &**b)).collect();
            let body: Option<String> = $crate::__esi_body!($($body_param)?);
            self.esi
                .query($method, $visibility, &path, Some(&params), body.as_deref())
                .await
        }
    };
}

/// Create a function for calling a single endpoint
/// with a POST request.
///
/// Follows the structure of the `api_get!` macro, with the
/// addition of taking an additional pair of `ident` and `ty`
/// to name and type the data that will be passed to
/// `serde_json::to_string` for serializing for setting the
/// request's body.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_post;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_post!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Public,
///         Vec<u64>,
///         (alliance_id: i64) => "{alliance_id}",
///         ids: &[u64],
///     );
///
/// }
/// # fn main() {}
/// ```
/// ## Result:
///
/// ```rust,ignore
/// /// Docs for the generated function
/// pub async fn function_name(&self, alliance_id: i64, ids: &[u64]) -> EsiResult<Vec<u64>> {
///     let path = self.esi.get_endpoint_for_op_id("some_operation_id")?
///         .replace("{alliance_id}", &alliance_id.to_string());
///     let body = serde_json::to_string(ids);
///     self.esi
///         .query("GET", RequestType::Public, &path, None, Some(&body))
///         .await
/// }
/// ```
#[macro_export]
macro_rules! api_post {
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*,
        $body_param:ident: $param_type:ty,
    ) => {
        $(#[$m])*
        pub async fn $fn_name(&self, $( $param: $param_t, )* $body_param: $param_type) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            let body = serde_json::to_string($body_param)?;
            self.esi.
                query("POST", $visibility, &path, None, Some(&body))
                .await
        }
    };
    // A POST whose body is an array with a maximum length: the slice is split into
    // requests of at most `$max` items and the answers are joined in order.
    // `path params ; Chunked(body: &[T], max)`.
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; Chunked($body_param:ident: $body_t:ty, $max:literal) $(,)?
    ) => {
        $(#[$m])*
        ///
        /// The spec allows at most
        #[doc = concat!(stringify!($max), " items per request; longer lists are split into several requests,")]
        /// sent concurrently, and the results are joined in order. An empty list sends nothing.
        pub async fn $fn_name(&self, $( $param: $param_t, )* $body_param: $body_t) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            self.esi
                .post_chunked($visibility, &path, $body_param, $max)
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; $( $qkind:ident($qparam:ident: $qparam_t:ty) => $qkey:literal ),*
        ; $body_param:ident: $body_t:ty $(,)?
    ) => {
        $crate::__esi_endpoint!(
            $(#[$m])*
            $fn_name, $op_id, $visibility, $ret_type, "POST",
            [ $( ($param: $param_t) => $replace ),* ]
            [ $( $qkind($qparam: $qparam_t) => $qkey ),* ]
            [ $body_param: $body_t ]
        );
    };
    // A POST request without a body: `path params ; NoBody`.
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; NoBody $(,)?
    ) => {
        $crate::__esi_endpoint!(
            $(#[$m])*
            $fn_name, $op_id, $visibility, $ret_type, "POST",
            [ $( ($param: $param_t) => $replace ),* ]
            [ ]
            [ ]
        );
    };
}

/// Create a function for calling a single endpoint
/// with a PUT request.
///
/// Follows the structure of the `api_post!` macro: path parameters,
/// followed by the data that will be passed to `serde_json::to_string`
/// to set the request's body.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_put;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_put!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Authenticated,
///         (),
///         (fleet_id: i32) => "{fleet_id}",
///         settings: &serde_json::Value,
///     );
///
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! api_put {
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*,
        $body_param:ident: $param_type:ty,
    ) => {
        $(#[$m])*
        pub async fn $fn_name(&self, $( $param: $param_t, )* $body_param: $param_type) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            let body = serde_json::to_string($body_param)?;
            self.esi.
                query("PUT", $visibility, &path, None, Some(&body))
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; $( $qkind:ident($qparam:ident: $qparam_t:ty) => $qkey:literal ),*
        ; $body_param:ident: $body_t:ty $(,)?
    ) => {
        $crate::__esi_endpoint!(
            $(#[$m])*
            $fn_name, $op_id, $visibility, $ret_type, "PUT",
            [ $( ($param: $param_t) => $replace ),* ]
            [ $( $qkind($qparam: $qparam_t) => $qkey ),* ]
            [ $body_param: $body_t ]
        );
    };
}

/// Create a function for calling a single endpoint
/// with a DELETE request.
///
/// Follows the structure of the `api_get!` macro: path parameters,
/// then required and optional query parameters. DELETE requests
/// have no body.
///
/// # Example
///
/// ```rust,no_run
/// # use esi_openapi::prelude::*;
/// # use esi_openapi::api_delete;
/// pub struct SomeGroup<'a> {
///     pub(crate) esi: &'a Esi,
/// }
///
/// impl SomeGroup<'_> {
///
///     api_delete!(
///         /// Docs for the generated function
///         function_name,
///         "some_operation_id",
///         RequestType::Authenticated,
///         (),
///         (character_id: i64) => "{character_id}",
///         (fitting_id: i32) => "{fitting_id}"
///     );
///
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! api_delete {
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
    ) => {
        $(#[$m])*
        pub async fn $fn_name(&self, $( $param: $param_t, )*) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            self.esi.
                query("DELETE", $visibility, &path, None, None)
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        $( ; $( ($qparam:ident: $qparam_t:ty) => $qreplace:literal ),+ )?
    ) => {
        $(#[$m])*
        pub async fn $fn_name(
            &self,
            $( $param: $param_t, )*
            $($( $qparam: $qparam_t, )*)?
        ) -> EsiResult<$ret_type> {
            let path = self
                .esi
                .get_endpoint_for_op_id($op_id)?
                $(
                    .replace($replace, &$param.to_string())
                )*;
            let params = vec![
                $($(
                    ($qreplace, $qparam.to_string()),
                )+)?
            ];
            let params: Vec<(&str, &str)> = params.iter().map(|(a, b)| (*a, &**b)).collect();
            self.esi.
                query("DELETE", $visibility, &path, Some(&params), None)
                .await
        }
    };
    (
        $(#[$m:meta])*
        $fn_name:ident,
        $op_id:literal,
        $visibility:expr,
        $ret_type:ty,
        $( ($param:ident: $param_t:ty) => $replace:literal ),*
        ; $( $qkind:ident($qparam:ident: $qparam_t:ty) => $qkey:literal ),+ $(,)?
    ) => {
        $crate::__esi_endpoint!(
            $(#[$m])*
            $fn_name, $op_id, $visibility, $ret_type, "DELETE",
            [ $( ($param: $param_t) => $replace ),* ]
            [ $( $qkind($qparam: $qparam_t) => $qkey ),+ ]
            [ ]
        );
    };
}
