-- A farm can produce several distinct things in the same month (eggs AND
-- meat, for instance) - the old UNIQUE(exploitation_id, mois) forced every
-- resubmission to overwrite whatever was declared first that month,
-- regardless of what it was. Scope uniqueness to (exploitation_id, mois, nom)
-- instead: resubmitting the same nom for the same month still overwrites (as
-- before), a different nom now creates its own row.
ALTER TABLE production DROP CONSTRAINT production_exploitation_id_mois_key;
ALTER TABLE production ADD CONSTRAINT production_exploitation_id_mois_nom_key
    UNIQUE (exploitation_id, mois, nom);
