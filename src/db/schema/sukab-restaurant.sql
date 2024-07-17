-- begin: create database
create database sukab_restaurant;
-- end: create database

-- begin: create tables
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

create table public.menus
(
    menu_id bigserial
        constraint menus_pk
            primary key,
    name    text
);
-- end: create tables

-- begin: master data for menus table

INSERT INTO public.menus (menu_id, name) VALUES (1, 'ちゃづけ');
INSERT INTO public.menus (menu_id, name) VALUES (2, 'らーめん');
INSERT INTO public.menus (menu_id, name) VALUES (3, '弁当');
INSERT INTO public.menus (menu_id, name) VALUES (4, '牛丼');
INSERT INTO public.menus (menu_id, name) VALUES (5, '焼き鳥');
INSERT INTO public.menus (menu_id, name) VALUES (6, '枝豆');
INSERT INTO public.menus (menu_id, name) VALUES (7, '刺身');
INSERT INTO public.menus (menu_id, name) VALUES (8, 'うどん');
INSERT INTO public.menus (menu_id, name) VALUES (9, 'Nasi Goreng');
INSERT INTO public.menus (menu_id, name) VALUES (10, 'Rendang');

-- end: master data for menus table
