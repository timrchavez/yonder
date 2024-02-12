BEGIN;


CREATE TABLE IF NOT EXISTS public."User"
(
    id uuid NOT NULL DEFAULT uuid_generate_v4(),
    first_name text COLLATE pg_catalog."default" NOT NULL,
    family_name text COLLATE pg_catalog."default" NOT NULL,
    country text COLLATE pg_catalog."default" NOT NULL,
    is_superuser boolean NOT NULL DEFAULT false,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now(),
    CONSTRAINT "User_pkey" PRIMARY KEY (id)
);

COMMENT ON TABLE public."User"
    IS 'Contains essential information about Yonder users.';

CREATE TABLE IF NOT EXISTS public."UserGroup"
(
    id uuid NOT NULL DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL,
    name text COLLATE pg_catalog."default" NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now(),
    CONSTRAINT "UserGroup_pkey" PRIMARY KEY (id)
);

COMMENT ON TABLE public."UserGroup"
    IS 'The Group users are in determine what they can do within Yonder.';

CREATE TABLE IF NOT EXISTS public."UserProfile"
(
    id uuid NOT NULL DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now(),
    CONSTRAINT "UserProfile_pkey" PRIMARY KEY (id),
    CONSTRAINT "UserProfile_User_unique" UNIQUE (user_id)
);

COMMENT ON TABLE public."UserProfile"
    IS 'Additional, non-essential information associated with the User.';

CREATE TABLE IF NOT EXISTS public._sqlx_migrations
(
    version bigint NOT NULL,
    description text COLLATE pg_catalog."default" NOT NULL,
    installed_on timestamp with time zone NOT NULL DEFAULT now(),
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL,
    CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version)
);

CREATE TABLE IF NOT EXISTS public."Account"
(
    email text NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (email)
);

ALTER TABLE IF EXISTS public."UserGroup"
    ADD CONSTRAINT "UserGroup_User_fkey" FOREIGN KEY (user_id)
    REFERENCES public."User" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

COMMENT ON CONSTRAINT "UserGroup_User_fkey" ON public."UserGroup"
    IS 'A User in the Group';



ALTER TABLE IF EXISTS public."UserProfile"
    ADD CONSTRAINT "UserProfile_User_fkey" FOREIGN KEY (user_id)
    REFERENCES public."User" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public."Account"
    ADD CONSTRAINT "Account_User_fkey" FOREIGN KEY (user_id)
    REFERENCES public."User" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

END;
