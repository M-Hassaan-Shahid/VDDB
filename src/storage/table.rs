#[derive(Debug)]
pub struct TableStore {
    pub table: Table,
    pub metadata: TableMetadata,
    pub data_dir: String,
    pub file_path: String,
}

#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub table_id: u64,
    pub row_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub columns: Vec<ColumnMetadata>,
}

#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub column_id: u64,
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<String>,
    pub indexes: Vec<Index>,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub index_type: IndexType,
}

#[derive(Debug, Clone)]
pub enum IndexType {
    BTree,
    Hash,
    FullText,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub columns: Vec<String>,
    pub check_condition: Option<String>,
    pub reference_table: Option<String>,
    pub reference_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum ConstraintType {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    NotNull,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
    Timestamp,
    Json,
    Binary,
    Array(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Timestamp(u64),
    Json(serde_json::Value),
    Binary(Vec<u8>),
    Array(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Null,
}
// ... rest of the implementation ... 