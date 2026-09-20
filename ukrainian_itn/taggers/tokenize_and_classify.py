import pynini
from pynini.lib import pynutil

from ukrainian_itn.graph_utils import GraphFst, delete_extra_space, delete_space
from ukrainian_itn.taggers.address import AddressFst
from ukrainian_itn.taggers.cardinal import CardinalFst
from ukrainian_itn.taggers.century import CenturyFst
from ukrainian_itn.taggers.code import CodeFst
from ukrainian_itn.taggers.date import DateFst
from ukrainian_itn.taggers.decade import DecadeFst
from ukrainian_itn.taggers.decimal import DecimalFst
from ukrainian_itn.taggers.duration import DurationFst
from ukrainian_itn.taggers.electronic import ElectronicFst
from ukrainian_itn.taggers.fraction import FractionFst
from ukrainian_itn.taggers.ip import IpFst
from ukrainian_itn.taggers.legal import LegalFst
from ukrainian_itn.taggers.measure import MeasureFst
from ukrainian_itn.taggers.money import MoneyFst
from ukrainian_itn.taggers.number_sign import NumberSignFst
from ukrainian_itn.taggers.ordinal import OrdinalFst
from ukrainian_itn.taggers.range import RangeFst
from ukrainian_itn.taggers.score import ScoreFst
from ukrainian_itn.taggers.telephone import TelephoneFst
from ukrainian_itn.taggers.time import TimeFst
from ukrainian_itn.taggers.version import VersionFst
from ukrainian_itn.taggers.whitelist import WhitelistFst
from ukrainian_itn.taggers.word import WordFst


class ClassifyFst(GraphFst):

    def __init__(self):
        super().__init__(name="tokenize_and_classify", kind="classify")

        cardinal = CardinalFst()
        cardinal_graph = cardinal.fst

        ordinal = OrdinalFst(cardinal)
        ordinal_graph = ordinal.fst

        decimal = DecimalFst(cardinal)
        decimal_graph = decimal.fst

        fraction_graph = FractionFst(cardinal=cardinal).fst
        measure_graph = MeasureFst(cardinal=cardinal, decimal=decimal).fst
        date_graph = DateFst(cardinal=cardinal, ordinal=ordinal).fst
        time_graph = TimeFst(cardinal=cardinal, ordinal=ordinal).fst
        telephone_graph = TelephoneFst(cardinal=cardinal).fst
        electronic_graph = ElectronicFst(cardinal=cardinal).fst
        century_graph = CenturyFst(cardinal=cardinal).fst
        number_sign_graph = NumberSignFst(cardinal=cardinal).fst
        range_graph = RangeFst(cardinal=cardinal).fst
        code_graph = CodeFst(cardinal=cardinal).fst
        address_graph = AddressFst(cardinal=cardinal).fst
        duration_graph = DurationFst(cardinal=cardinal).fst
        decade_graph = DecadeFst().fst
        legal_graph = LegalFst(cardinal=cardinal).fst
        score_graph = ScoreFst(cardinal=cardinal).fst
        version_graph = VersionFst(cardinal=cardinal).fst
        ip_graph = IpFst(cardinal=cardinal).fst
        word_graph = WordFst().fst
        money_graph = MoneyFst(cardinal=cardinal, decimal=decimal).fst
        whitelist_graph = WhitelistFst().fst

        classify = (
                pynutil.add_weight(whitelist_graph, 1.01)
                | pynutil.add_weight(duration_graph, 1.09)
                | pynutil.add_weight(decade_graph, 1.09)
                | pynutil.add_weight(legal_graph, 1.09)
                | pynutil.add_weight(score_graph, 1.09)
                | pynutil.add_weight(version_graph, 1.09)
                | pynutil.add_weight(ip_graph, 1.09)
                | pynutil.add_weight(electronic_graph, 1.09)
                | pynutil.add_weight(address_graph, 1.09)
                | pynutil.add_weight(range_graph, 1.09)
                | pynutil.add_weight(century_graph, 1.09)
                | pynutil.add_weight(number_sign_graph, 1.09)
                | pynutil.add_weight(code_graph, 1.09)
                | pynutil.add_weight(telephone_graph, 1.09)
                | pynutil.add_weight(decimal_graph, 1.1)
                | pynutil.add_weight(fraction_graph, 1.1)
                | pynutil.add_weight(measure_graph, 1.1)
                | pynutil.add_weight(cardinal_graph, 1.1)
                | pynutil.add_weight(ordinal_graph, 1.1)
                | pynutil.add_weight(money_graph, 1.1)
                | pynutil.add_weight(date_graph, 1.1)
                | pynutil.add_weight(time_graph, 1.1)
                | pynutil.add_weight(word_graph, 100)
        )

        token = pynutil.insert("tokens { ") + classify + pynutil.insert(" }")

        graph = token + pynini.closure(delete_extra_space + token)
        graph = delete_space + graph + delete_space

        self.fst = graph.optimize()
