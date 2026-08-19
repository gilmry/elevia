-- Production had no way to say *what* was produced (just a bare quantity +
-- free-text unit), and the estimated margin implicitly assumed everything
-- produced in a month was also sold that same month at the declared price -
-- rarely true (stock carried over, partial sales, spoilage...).
ALTER TABLE production
    ADD COLUMN nom TEXT NOT NULL DEFAULT '',
    ADD COLUMN quantite_vendue NUMERIC(14, 3) CHECK (quantite_vendue >= 0);

-- Only future inserts are required to supply a name; existing rows keep the
-- '' backfilled above rather than being deleted.
ALTER TABLE production ALTER COLUMN nom DROP DEFAULT;
