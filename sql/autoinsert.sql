CREATE OR REPLACE FUNCTION public.insert_into_entities()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO public.entities(id) VALUES (NEW.id)
    ON CONFLICT DO NOTHING; -- Prevents duplicate insert if id already exists
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;