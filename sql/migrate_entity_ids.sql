-- Insert existing IDs into entities from stories
INSERT INTO public.entities (id)
SELECT id FROM public.stories
ON CONFLICT DO NOTHING;

-- Insert existing IDs into entities from characters
INSERT INTO public.entities (id)
SELECT id FROM public.characters
ON CONFLICT DO NOTHING;

-- Insert existing IDs into entities from locations
INSERT INTO public.entities (id)
SELECT id FROM public.locations
ON CONFLICT DO NOTHING;

-- Insert existing IDs into entities from fragments
INSERT INTO public.entities (id)
SELECT id FROM public.fragments
ON CONFLICT DO NOTHING;