ALTER TABLE public.characters
    ADD CONSTRAINT characters_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);

ALTER TABLE public.fragments
    ADD CONSTRAINT fragments_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);

ALTER TABLE public.locations
    ADD CONSTRAINT locations_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);

ALTER TABLE public.stories
    ADD CONSTRAINT stories_entity_fkey FOREIGN KEY (id) REFERENCES public.entities(id);