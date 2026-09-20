from pynini.lib import pynutil

from ukrainian_itn.graph_utils import GraphFst
from ukrainian_itn.verbalizers.cardinal import CardinalFst
from ukrainian_itn.verbalizers.date import DateFst
from ukrainian_itn.verbalizers.decimal import DecimalFst
from ukrainian_itn.verbalizers.electronic import ElectronicFst
from ukrainian_itn.verbalizers.fraction import FractionFst
from ukrainian_itn.verbalizers.measure import MeasureFst
from ukrainian_itn.verbalizers.money import MoneyFst
from ukrainian_itn.verbalizers.ordinal import OrdinalFst
from ukrainian_itn.verbalizers.telephone import TelephoneFst
from ukrainian_itn.verbalizers.time import TimeFst
from ukrainian_itn.verbalizers.whitelist import WhitelistFst
from ukrainian_itn.verbalizers.word import WordFst


class VerbalizeFst(GraphFst):

    def __init__(self):
        super().__init__(name="verbalize", kind="verbalize")

        self.cardinal = CardinalFst()
        self.decimal = DecimalFst()
        self.ordinal = OrdinalFst()
        self.fraction = FractionFst()
        self.telephone = TelephoneFst()
        self.electronic = ElectronicFst()
        self.measure = MeasureFst(decimal=self.decimal, cardinal=self.cardinal)
        self.money = MoneyFst(decimal=self.decimal)
        self.date = DateFst()
        self.time = TimeFst()
        self.word = WordFst()
        self.whitelist = WhitelistFst()

        graph = (
                self.whitelist.fst
                | self.time.fst
                | self.date.fst
                | self.money.fst
                | self.measure.fst
                | self.ordinal.fst
                | self.fraction.fst
                | self.telephone.fst
                | self.electronic.fst
                | self.decimal.fst
                | self.cardinal.fst
        )
        graph |= pynutil.add_weight(self.word.fst, 100)

        self.fst = graph

    def as_json(self):
        graph = (
                self.whitelist.as_json()
                | self.money.as_json()
                | self.measure.as_json()
                | self.time.as_json()
                | self.date.as_json()
                | self.ordinal.as_json()
                | self.fraction.as_json()
                | self.telephone.as_json()
                | self.electronic.as_json()
                | self.decimal.as_json()
                | self.cardinal.as_json()
        )

        graph |= pynutil.add_weight(self.word.as_json(), 100)

        return graph
