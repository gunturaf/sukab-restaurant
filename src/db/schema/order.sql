create database sukab_restaurant;

create table public.orders
(
    order_id     bigserial
        constraint orders_pk
            primary key,
    menu_id      integer,
    table_number integer,
    cook_time    integer,
    created_at   timestamp with time zone
);

