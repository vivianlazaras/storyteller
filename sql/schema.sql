--
-- PostgreSQL database dump
--

-- Dumped from database version 15.9
-- Dumped by pg_dump version 15.9

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: storyteller
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO storyteller;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: storyteller
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO storyteller;

--
-- Name: insert_into_entities(); Type: FUNCTION; Schema: public; Owner: storyteller
--

CREATE FUNCTION public.insert_into_entities() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO public.entities(id) VALUES (NEW.id)
    ON CONFLICT DO NOTHING; -- Prevents duplicate insert if id already exists
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.insert_into_entities() OWNER TO storyteller;

--
-- Name: unix_now(); Type: FUNCTION; Schema: public; Owner: storyteller
--

CREATE FUNCTION public.unix_now() RETURNS bigint
    LANGUAGE plpgsql IMMUTABLE
    AS $$
BEGIN
    RETURN EXTRACT(EPOCH FROM now())::BIGINT;
END;
$$;


ALTER FUNCTION public.unix_now() OWNER TO storyteller;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: characters; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.characters (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    timeline uuid,
    name text NOT NULL,
    description text,
    metadata uuid,
    created bigint DEFAULT public.unix_now(),
    last_edited bigint DEFAULT public.unix_now(),
    image text
);


ALTER TABLE public.characters OWNER TO storyteller;

--
-- Name: edits; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.edits (
    id uuid NOT NULL,
    date bigint,
    editor uuid,
    comment text,
    value text NOT NULL,
    prevvalue text,
    change character varying(2)
);


ALTER TABLE public.edits OWNER TO storyteller;

--
-- Name: entities; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.entities (
    id uuid NOT NULL,
    active boolean DEFAULT true
);


ALTER TABLE public.entities OWNER TO storyteller;

--
-- Name: exif_tags; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.exif_tags (
    image uuid NOT NULL,
    tag text NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.exif_tags OWNER TO storyteller;

--
-- Name: fragments; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.fragments (
    id uuid NOT NULL,
    metadata uuid,
    idx integer NOT NULL,
    content text NOT NULL,
    name text NOT NULL,
    last_edited bigint DEFAULT public.unix_now(),
    created bigint DEFAULT public.unix_now(),
    image text
);


ALTER TABLE public.fragments OWNER TO storyteller;

--
-- Name: grouprel; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.grouprel (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    groupid uuid,
    userid uuid,
    description text
);


ALTER TABLE public.grouprel OWNER TO storyteller;

--
-- Name: images; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.images (
    id uuid NOT NULL,
    metadata uuid,
    url text NOT NULL,
    description text
);


ALTER TABLE public.images OWNER TO storyteller;

--
-- Name: licenses; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.licenses (
    id uuid NOT NULL,
    name text NOT NULL,
    description text,
    public boolean DEFAULT false,
    content text
);


ALTER TABLE public.licenses OWNER TO storyteller;

--
-- Name: locations; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.locations (
    id uuid NOT NULL,
    timeline uuid,
    name text NOT NULL,
    description text,
    metadata uuid,
    created bigint DEFAULT public.unix_now(),
    last_edited bigint DEFAULT public.unix_now()
);


ALTER TABLE public.locations OWNER TO storyteller;

--
-- Name: metadata; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.metadata (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    creator uuid,
    license uuid,
    shared uuid,
    public boolean DEFAULT false,
    active boolean DEFAULT true
);


ALTER TABLE public.metadata OWNER TO storyteller;

--
-- Name: relations; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.relations (
    parent uuid NOT NULL,
    child uuid NOT NULL,
    description text,
    parent_category text NOT NULL,
    child_category text NOT NULL
);


ALTER TABLE public.relations OWNER TO storyteller;

--
-- Name: stories; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.stories (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    timeline uuid,
    name text NOT NULL,
    description text,
    renderer text,
    metadata uuid,
    created bigint DEFAULT public.unix_now(),
    last_edited bigint DEFAULT public.unix_now(),
    image text
);


ALTER TABLE public.stories OWNER TO storyteller;

--
-- Name: tags; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.tags (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    value text NOT NULL,
    entity uuid
);


ALTER TABLE public.tags OWNER TO storyteller;

--
-- Name: tasks; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.tasks (
    id uuid NOT NULL,
    name text NOT NULL,
    description text,
    created bigint DEFAULT public.unix_now(),
    completed bigint,
    deadline bigint
);


ALTER TABLE public.tasks OWNER TO storyteller;

--
-- Name: timelines; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.timelines (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    metadata uuid,
    created bigint,
    last_edited bigint
);


ALTER TABLE public.timelines OWNER TO storyteller;

--
-- Name: universe; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.universe (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    description text,
    metadata uuid,
    created bigint,
    last_edited bigint
);


ALTER TABLE public.universe OWNER TO storyteller;

--
-- Name: usergroups; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.usergroups (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text,
    description text
);


ALTER TABLE public.usergroups OWNER TO storyteller;

--
-- Name: users; Type: TABLE; Schema: public; Owner: storyteller
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    fname text NOT NULL,
    lname text NOT NULL,
    subject uuid,
    email text NOT NULL
);


ALTER TABLE public.users OWNER TO storyteller;

--
-- Name: characters characters_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.characters
    ADD CONSTRAINT characters_pkey PRIMARY KEY (id);


--
-- Name: edits edits_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.edits
    ADD CONSTRAINT edits_pkey PRIMARY KEY (id);


--
-- Name: entities entities_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.entities
    ADD CONSTRAINT entities_pkey PRIMARY KEY (id);


--
-- Name: exif_tags exif_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.exif_tags
    ADD CONSTRAINT exif_tags_pkey PRIMARY KEY (image, tag);


--
-- Name: fragments fragments_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.fragments
    ADD CONSTRAINT fragments_pkey PRIMARY KEY (id);


--
-- Name: grouprel grouprel_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.grouprel
    ADD CONSTRAINT grouprel_pkey PRIMARY KEY (id);


--
-- Name: images images_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_pkey PRIMARY KEY (id);


--
-- Name: imagetags imagetags_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.imagetags
    ADD CONSTRAINT imagetags_pkey PRIMARY KEY (id);


--
-- Name: licenses licenses_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.licenses
    ADD CONSTRAINT licenses_pkey PRIMARY KEY (id);


--
-- Name: locations locations_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.locations
    ADD CONSTRAINT locations_pkey PRIMARY KEY (id);


--
-- Name: metadata metadata_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.metadata
    ADD CONSTRAINT metadata_pkey PRIMARY KEY (id);


--
-- Name: relations relations_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.relations
    ADD CONSTRAINT relations_pkey PRIMARY KEY (parent, child, parent_category, child_category);


--
-- Name: stories stories_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.stories
    ADD CONSTRAINT stories_pkey PRIMARY KEY (id);


--
-- Name: tags tags_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_pkey PRIMARY KEY (id);


--
-- Name: tasks tasks_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_pkey PRIMARY KEY (id);


--
-- Name: timelines timelines_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.timelines
    ADD CONSTRAINT timelines_pkey PRIMARY KEY (id);


--
-- Name: universe universe_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.universe
    ADD CONSTRAINT universe_pkey PRIMARY KEY (id);


--
-- Name: usergroups usergroups_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.usergroups
    ADD CONSTRAINT usergroups_pkey PRIMARY KEY (id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: characters characters_insert_entity; Type: TRIGGER; Schema: public; Owner: storyteller
--

CREATE TRIGGER characters_insert_entity BEFORE INSERT ON public.characters FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();


--
-- Name: fragments fragments_insert_entity; Type: TRIGGER; Schema: public; Owner: storyteller
--

CREATE TRIGGER fragments_insert_entity BEFORE INSERT ON public.fragments FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();


--
-- Name: locations locations_insert_entity; Type: TRIGGER; Schema: public; Owner: storyteller
--

CREATE TRIGGER locations_insert_entity BEFORE INSERT ON public.locations FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();


--
-- Name: stories stories_insert_entity; Type: TRIGGER; Schema: public; Owner: storyteller
--

CREATE TRIGGER stories_insert_entity BEFORE INSERT ON public.stories FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();


--
-- Name: tasks tasks_insert_entity; Type: TRIGGER; Schema: public; Owner: storyteller
--

CREATE TRIGGER tasks_insert_entity BEFORE INSERT ON public.tasks FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();


--
-- Name: characters characters_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.characters
    ADD CONSTRAINT characters_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: characters characters_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.characters
    ADD CONSTRAINT characters_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: characters characters_timeline_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.characters
    ADD CONSTRAINT characters_timeline_fkey FOREIGN KEY (timeline) REFERENCES public.timelines(id);


--
-- Name: edits edits_editor_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.edits
    ADD CONSTRAINT edits_editor_fkey FOREIGN KEY (editor) REFERENCES public.users(id);


--
-- Name: exif_tags exif_tags_image_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.exif_tags
    ADD CONSTRAINT exif_tags_image_fkey FOREIGN KEY (image) REFERENCES public.images(id);


--
-- Name: fragments fragments_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.fragments
    ADD CONSTRAINT fragments_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: fragments fragments_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.fragments
    ADD CONSTRAINT fragments_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: grouprel grouprel_groupid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.grouprel
    ADD CONSTRAINT grouprel_groupid_fkey FOREIGN KEY (groupid) REFERENCES public.usergroups(id);


--
-- Name: grouprel grouprel_userid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.grouprel
    ADD CONSTRAINT grouprel_userid_fkey FOREIGN KEY (userid) REFERENCES public.users(id);


--
-- Name: images images_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: images images_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.images
    ADD CONSTRAINT images_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: imagetags imagetags_image_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.imagetags
    ADD CONSTRAINT imagetags_image_fkey FOREIGN KEY (image) REFERENCES public.images(id);


--
-- Name: locations locations_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.locations
    ADD CONSTRAINT locations_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: locations locations_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.locations
    ADD CONSTRAINT locations_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: locations locations_timeline_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.locations
    ADD CONSTRAINT locations_timeline_fkey FOREIGN KEY (timeline) REFERENCES public.timelines(id);


--
-- Name: metadata metadata_creator_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.metadata
    ADD CONSTRAINT metadata_creator_fkey FOREIGN KEY (creator) REFERENCES public.users(id);


--
-- Name: metadata metadata_license_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.metadata
    ADD CONSTRAINT metadata_license_fkey FOREIGN KEY (license) REFERENCES public.licenses(id);


--
-- Name: metadata metadata_shared_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.metadata
    ADD CONSTRAINT metadata_shared_fkey FOREIGN KEY (shared) REFERENCES public.usergroups(id);


--
-- Name: relations relations_child_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.relations
    ADD CONSTRAINT relations_child_fkey FOREIGN KEY (child) REFERENCES public.entities(id);


--
-- Name: relations relations_parent_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.relations
    ADD CONSTRAINT relations_parent_fkey FOREIGN KEY (parent) REFERENCES public.entities(id);


--
-- Name: stories stories_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.stories
    ADD CONSTRAINT stories_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: stories stories_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.stories
    ADD CONSTRAINT stories_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: stories stories_timeline_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.stories
    ADD CONSTRAINT stories_timeline_fkey FOREIGN KEY (timeline) REFERENCES public.timelines(id);


--
-- Name: tags tags_entity_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT tags_entity_fkey FOREIGN KEY (entity) REFERENCES public.entities(id);


--
-- Name: tasks tasks_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.tasks
    ADD CONSTRAINT tasks_id_fkey FOREIGN KEY (id) REFERENCES public.entities(id);


--
-- Name: timelines timelines_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.timelines
    ADD CONSTRAINT timelines_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: universe universe_metadata_fkey; Type: FK CONSTRAINT; Schema: public; Owner: storyteller
--

ALTER TABLE ONLY public.universe
    ADD CONSTRAINT universe_metadata_fkey FOREIGN KEY (metadata) REFERENCES public.metadata(id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: pg_database_owner
--

GRANT ALL ON SCHEMA public TO storyteller;


--
-- PostgreSQL database dump complete
--

