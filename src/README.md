# Creating the Database


create table sensor_metadata (
id UUID Primary Key,
name VarChar(30),
geohash VarChar(12));


create table sensor_data (
time TIMESTAMPTZ,
id uuid,
rssi integer,
primary key (time, id),
foreign key (id) references sensor_metadata(id) on delete cascade);
