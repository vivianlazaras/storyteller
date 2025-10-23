CREATE TRIGGER characters_insert_entity
BEFORE INSERT ON public.characters
FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();

CREATE TRIGGER fragments_insert_entity
BEFORE INSERT ON public.fragments
FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();

CREATE TRIGGER locations_insert_entity
BEFORE INSERT ON public.locations
FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();

CREATE TRIGGER stories_insert_entity
BEFORE INSERT ON public.stories
FOR EACH ROW EXECUTE FUNCTION public.insert_into_entities();