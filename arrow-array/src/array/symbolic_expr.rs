use arrow_schema::DataType;

/// Literal values used in symbolic expressions
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    /// represents `DataType::Null` (castable to/from any other type)
    Null,
    /// true or false value
    Boolean(Option<bool>),
    /// 32bit float
    Float32(Option<f32>),
    /// 64bit float
    Float64(Option<f64>),
    /// 128bit decimal, using the i128 to represent the decimal, precision scale
    Decimal128(Option<i128>, u8, i8),
    /// signed 8bit int
    Int8(Option<i8>),
    /// signed 16bit int
    Int16(Option<i16>),
    /// signed 32bit int
    Int32(Option<i32>),
    /// signed 64bit int
    Int64(Option<i64>),
    /// unsigned 8bit int
    UInt8(Option<u8>),
    /// unsigned 16bit int
    UInt16(Option<u16>),
    /// unsigned 32bit int
    UInt32(Option<u32>),
    /// unsigned 64bit int
    UInt64(Option<u64>),
    /// utf-8 encoded string.
    Utf8(Option<String>),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VarType {
    /// A boolean variable
    Boolean,
    /// A string variable
    String,
    /// A int variable
    Int,
}

pub fn arrow_type_to_var_type(arrow_type: DataType) -> VarType {
    match arrow_type {
        DataType::Boolean => VarType::Boolean,
        DataType::Utf8 | DataType::LargeUtf8 => VarType::String,
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Decimal128(_, _)
        | DataType::Decimal256(_, _) => VarType::Int,
        _ => VarType::String, // Default to String for other types
    }
}

/// Operators applied to symbolic expressions
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum Operator {
    /// Expressions are equal
    Equal,
    /// Expressions are not equal
    NotEqual,
    /// Left side is smaller than right side
    Less,
    /// Left side is smaller or equal to right side
    LessEqual,
    /// Left side is greater than right side
    Greater,
    /// Left side is greater or equal to right side
    GreaterEqual,
    /// Addition
    Plus,
    /// Subtraction
    Minus,
    /// Multiplication operator, like `*`
    Multiply,
    /// Division operator, like `/`
    Divide,
    /// Remainder operator, like `%`
    Modulo,
    /// Logical AND, like `&&`
    And,
    /// Logical OR, like `||`
    Or,
    /// Logical NOT, like `!`
    Not,
    /// In, like `in`
    In,
}

/// A symbolic expression for a columnar array.
///
/// This is used to represent the symbolic data of an array.
/// Used to represent the symbolic data of an array.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal symbolic expression
    Literal(ScalarValue),

    /// A symbolic variable representing a cell in a table
    Variable {
        /// The name of the table
        table: String,
        /// The position of the column
        column: usize,
        /// The position of the row
        row: usize,
        /// The variable type
        var_type: VarType,
    },

    /// A symbolic variable representing a row in a table
    Row {
        /// The name of the table
        table: String,
        /// The position of the row
        row_id: usize,
        /// The row constraint
        constraint: Option<Vec<Expr>>,
    },

    /// A list of symbolic expressions
    List(Vec<Expr>),

    /// A count symbolic expression
    Count {
        /// The column to count
        expr: Box<Expr>,
        /// Whether the count is distinct
        is_distinct: bool,
    },

    /// A binary symbolic expression
    BinaryExpr {
        /// The left side of the binary expression
        left: Box<Expr>,
        /// The operator of the binary expression
        op: Operator,
        /// The right side of the binary expression
        right: Box<Expr>,
    },

    /// A negation symbolic expression  
    Not(Box<Expr>),

    /// A null check symbolic expression
    IsNull(Box<Expr>),

    /// A not null check symbolic expression
    IsNotNull(Box<Expr>),
}

impl Expr {
    /// Create a literal symbolic expression
    pub fn literal(val: ScalarValue) -> Self {
        Expr::Literal(val)
    }

    /// Create a symbolic variable
    pub fn variable(table: String, column: usize, row: usize, var_type: VarType) -> Self {
        Expr::Variable {
            table,
            column,
            row,
            var_type,
        }
    }

    /// Create a symbolic row
    pub fn row(table: String, row_id: usize) -> Self {
        Expr::Row {
            table,
            row_id,
            constraint: None,
        }
    }

    /// Create a symbolic row with a constraint
    pub fn row_with_constraint(table: String, row_id: usize, constraint: Vec<Expr>) -> Self {
        Expr::Row {
            table,
            row_id,
            constraint: Some(constraint),
        }
    }

    /// Create a count symbolic expression
    pub fn count(expr: Expr, is_distinct: bool) -> Self {
        Expr::Count {
            expr: Box::new(expr),
            is_distinct,
        }
    }

    /// Create a binary symbolic expression
    pub fn binary(left: Expr, op: Operator, right: Expr) -> Self {
        Expr::BinaryExpr {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// Create a negation symbolic expression
    pub fn not(expr: Expr) -> Self {
        Expr::Not(Box::new(expr))
    }

    /// Create a null check symbolic expression
    pub fn is_null(expr: Expr) -> Self {
        Expr::IsNull(Box::new(expr))
    }

    /// Create a not null check symbolic expression
    pub fn is_not_null(expr: Expr) -> Self {
        Expr::IsNotNull(Box::new(expr))
    }
}

/// Create a symbolic expression array for a column reference using table name and column position
pub fn make_colref_symbolic_expr_array(
    table: String,
    column: usize,
    len: usize,
    row_offset: usize,
    var_type: VarType,
) -> Vec<Expr> {
    let mut res = Vec::with_capacity(len);
    for i in 0..len {
        res.push(Expr::Variable {
            table: table.clone(),
            column,
            row: i + row_offset,
            var_type,
        });
    }
    res
}

pub fn add_constraint_to_row(row: &mut Expr, constraint: Expr) {
    if let Expr::Row {
        constraint: Some(constraints),
        ..
    } = row
    {
        constraints.push(constraint);
    } else if let Expr::Row {
        table,
        row_id,
        constraint: None,
    } = row
    {
        *row = Expr::Row {
            table: table.clone(),
            row_id: *row_id,
            constraint: Some(vec![constraint]),
        };
    } else {
        panic!("Expr is not a symbolic row");
    }
}
