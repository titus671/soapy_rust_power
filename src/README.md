# Creating the Database


create table sensor_metadata (
id UUID Primary Key,
name VarChar(30),
geohash VarChar(12));


create table sensor_data (
time TIMESTAMPTZ,
id uuid,
rssi integer,
frequency real,
primary key (time, frequency, id),
foreign key (id) references sensor_metadata(id) on delete cascade);


ALTER TABLE sensor_metadata 
ALTER COLUMN id SET DEFAULT gen_random_uuid();
