TARGET = main

LATEX = pdflatex

all: $(TARGET).pdf

$(TARGET).pdf: $(TARGET).tex
	$(LATEX) $(TARGET).tex
	$(LATEX) $(TARGET).tex

clean:
	rm -f *.aux *.log *.out *.toc *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz *.bbl *.blg *.nav *.snm *.vrb

cleanall: clean
	rm -f $(TARGET).pdf

.PHONY: all clean cleanall