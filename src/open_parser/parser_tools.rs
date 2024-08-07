
#[derive(Debug, Clone)]
pub struct OpenApiRequester {
    /// 接口说明
    pub summary: String,

    /// 调用接口名称
    pub operation_id: String,

    /// 接口调用地址
    pub url: String,

    /// 请求方式 目前支持 get post delete put
    pub method: String,

    pub request_schema_name: String,

    /// 请求参数类型
    /// 无论post 或 get 都将生成唯一的类型
    pub request_type_name: String,

    /// 请求的 content_type
    // pub content_type: String,

    pub response_type_name: String,

    /// 判断是否是form
    /// post请求， 有parameter则视为form
    pub is_form: bool
}

pub struct OpenApiModule {
    // tag 描述
    pub description: String,

    // tag 名称
    pub name: String,
}


