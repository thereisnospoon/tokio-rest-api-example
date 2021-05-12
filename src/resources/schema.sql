create table if not exists users (
    id varchar(100) primary key,
    name varchar(100),
    age int,
    sex varchar(1)
);

create index if not exists age_idx on users(age);
create index if not exists name_idx on users(name);