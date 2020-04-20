use sqlx::postgres::Postgres;
use sqlx_test::test_type;

// NULL
test_type!(null<Option<i16>>(Postgres,
    "NULL::int2" == None::<i16>
));

// boolean
test_type!(bool<bool>(Postgres,
    "false::boolean" == false,
    "true::boolean" == true
));

//
// character types
//

test_type!(str<String>(Postgres,
    "'this is foo'" == format!("this is foo"),
    "''" == ""
));

//
// numeric types
//

test_type!(i8(
    Postgres,
    "0::\"char\"" == 0_i8,
    "120::\"char\"" == 120_i8,
));

test_type!(i16(
    Postgres,
    "-2144::smallint" == -2144_i16,
    "821::smallint" == 821_i16,
));

test_type!(i32(
    Postgres,
    "94101::int" == 94101_i32,
    "-5101::int" == -5101_i32
));

test_type!(i64(Postgres, "9358295312::bigint" == 9358295312_i64));

test_type!(f32(Postgres, "9419.122::real" == 9419.122_f32));

test_type!(f64(
    Postgres,
    "939399419.1225182::double precision" == 939399419.1225182_f64
));
