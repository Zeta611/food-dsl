module Main exposing (main)

import Browser
import Draggable
import Html exposing (Html, aside, button, div, h2, input, label, node, p, span, text)
import Html.Attributes exposing (checked, class, classList, property, step, style, type_, value)
import Html.Events exposing (on, onCheck, onClick, onInput, onMouseEnter, onMouseLeave)
import Html.Lazy exposing (lazy6)
import Json.Decode as D
import Json.Encode as E


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Model =
    { cup : Cup
    , toolpath : Bool
    , infill : Maybe Lattice
    , spacing : Float
    , gesture : Gesture
    , hover : Maybe String
    , panelState : PanelState
    }


{-| "Set this parameter to this value", proposed by a gesture on the cup.
-}
type alias Candidate =
    { key : String
    , value : Float
    }


type Gesture
    = Idle
      -- the pointer is down on the cup; these are what it could mean right now
    | Dragging (List Candidate)
      -- the gesture ended but was ambiguous; waiting for a pick on the panel
    | Choosing (List Candidate)


{-| One number in the cup that can be changed by dragging the cup itself.
`path` tells the preview element where the number sits in the encoded cup,
so it must agree with `encodeCup`.
-}
type alias Param =
    { key : String
    , label : String
    , path : List String
    , min : Float
    , max : Float
    , step : Float
    , color : String
    , get : Cup -> Float
    , set : Float -> Cup -> Cup
    }


{-| Radii stay >= 5 and ripples <= 2 so that 5 - 2 - 2 > 0.
These ranges are also the ranges of the matching sliders on the panel.
-}
params : List Param
params =
    [ Param "bottomRadius" "Bottom radius" [ "bottomRadius" ] 5 40 0.5 "#2563eb" .bottomRadius (\v c -> { c | bottomRadius = v })
    , Param "topRadius" "Top radius" [ "topRadius" ] 5 50 0.5 "#db2777" .topRadius (\v c -> { c | topRadius = v })
    , Param "height" "Height" [ "height" ] 5 60 1 "#16a34a" .height (\v c -> { c | height = v })
    , Param "fluteDepth" "Flute depth" [ "flutes", "amplitude" ] 0 2 0.1 "#d97706" .fluteAmplitude (\v c -> { c | fluteAmplitude = v })
    , Param "ringDepth" "Ring depth" [ "rings", "amplitude" ] 0 2 0.1 "#7c3aed" .ringAmplitude (\v c -> { c | ringAmplitude = v })
    ]


findParam : String -> Maybe Param
findParam key =
    List.head (List.filter (\param -> param.key == key) params)


{-| Mockup "AST"
-}
type alias Cup =
    { bottomRadius : Float
    , topRadius : Float
    , bottom : Profile
    , top : Profile
    , height : Float
    , fluteAmplitude : Float
    , fluteCount : Int
    , ringAmplitude : Float
    , ringCount : Int
    }


{-| Must match `Profile` in `dsl/src/lib.rs`
-}
type Profile
    = Circle
    | Squircle


{-| Must match `LatticePreset` in `dsl/src/lib.rs`
-}
type Lattice
    = Grid
    | Honeycomb


type alias PanelState =
    { pos : ( Float, Float )
    , drag : Draggable.State ()
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { cup =
            { bottomRadius = 10
            , topRadius = 14
            , bottom = Circle
            , top = Circle
            , height = 16
            , fluteAmplitude = 0.7
            , fluteCount = 23
            , ringAmplitude = 0.5
            , ringCount = 1
            }
      , toolpath = False
      , infill = Nothing
      , spacing = 2
      , gesture = Idle
      , hover = Nothing
      , panelState =
            { pos = ( 0, 0 )
            , drag = Draggable.init
            }
      }
    , Cmd.none
    )



-- UPDATE


type Msg
    = CupSetBottom Profile
    | CupSetTop Profile
    | CupSetFluteCount Int
    | CupSetRingCount Int
    | SetToolpath Bool
    | SetInfill (Maybe Lattice)
    | SetSpacing Float
    | SetParam String Float
    | DragPreview (List Candidate)
    | DragEnd (List Candidate)
    | DragCancel
    | Choose Candidate
    | Hover (Maybe String)
    | PanelOnDragBy Draggable.Delta
    | PanelDragMsg (Draggable.Msg ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ cup, panelState } as model) =
    case msg of
        CupSetBottom p ->
            ( { model | cup = { cup | bottom = p } }, Cmd.none )

        CupSetTop p ->
            ( { model | cup = { cup | top = p } }, Cmd.none )

        CupSetFluteCount n ->
            ( { model | cup = { cup | fluteCount = n } }, Cmd.none )

        CupSetRingCount n ->
            ( { model | cup = { cup | ringCount = n } }, Cmd.none )

        SetToolpath w ->
            ( { model | toolpath = w }, Cmd.none )

        SetInfill l ->
            ( { model | infill = l }, Cmd.none )

        SetSpacing s ->
            ( { model | spacing = s }, Cmd.none )

        SetParam key v ->
            ( { model | cup = setParam key v cup }, Cmd.none )

        DragPreview candidates ->
            ( { model | gesture = Dragging candidates }, Cmd.none )

        DragEnd candidates ->
            -- Every parameter already has a slider, so a proposal that does
            -- not change the cup (a plain click) is not worth asking about.
            case List.filter (changesCup cup) candidates of
                [] ->
                    ( { model | gesture = Idle }, Cmd.none )

                [ only ] ->
                    ( commit only model, Cmd.none )

                several ->
                    ( { model | gesture = Choosing several }, Cmd.none )

        DragCancel ->
            ( { model | gesture = Idle, hover = Nothing }, Cmd.none )

        Choose candidate ->
            ( commit candidate model, Cmd.none )

        Hover key ->
            ( { model | hover = key }, Cmd.none )

        PanelOnDragBy ( dx, dy ) ->
            let
                ( x, y ) =
                    panelState.pos
            in
            ( { model
                | panelState =
                    { panelState
                        | pos = ( x + dx, y + dy )
                    }
              }
            , Cmd.none
            )

        PanelDragMsg dragMsg ->
            Tuple.mapFirst
                (\x -> { model | panelState = x })
                (Draggable.update dragConfig dragMsg panelState)


changesCup : Cup -> Candidate -> Bool
changesCup cup { key, value } =
    findParam key
        |> Maybe.map (\param -> param.get cup /= value)
        |> Maybe.withDefault False


setParam : String -> Float -> Cup -> Cup
setParam key v cup =
    findParam key
        |> Maybe.map (\param -> param.set v cup)
        |> Maybe.withDefault cup


{-| A finished gesture edits the cup. The panel already has every slider,
so unlike the empty-panel prototype, nothing is added to it.
-}
commit : Candidate -> Model -> Model
commit { key, value } model =
    { model | cup = setParam key value model.cup, gesture = Idle, hover = Nothing }


dragConfig : Draggable.Config () Msg
dragConfig =
    Draggable.basicConfig PanelOnDragBy



-- VIEW


view : Model -> Html Msg
view { cup, toolpath, infill, spacing, gesture, hover, panelState } =
    let
        ( x, y ) =
            panelState.pos

        panelTrans =
            "translate(" ++ String.fromFloat x ++ "px," ++ String.fromFloat y ++ "px)"

        dragHandle =
            Draggable.mouseTrigger () PanelDragMsg :: Draggable.touchTriggers () PanelDragMsg

        candidates =
            case gesture of
                Idle ->
                    []

                Dragging cs ->
                    cs

                Choosing cs ->
                    cs

        -- A slider for a draggable parameter. While a drag proposes a value
        -- for it, the slider shows that value and is highlighted.
        paramSlider key =
            case findParam key of
                Nothing ->
                    text ""

                Just param ->
                    let
                        proposed =
                            List.head (List.filter (\c -> c.key == key) candidates)
                    in
                    div
                        [ class "rounded-box -mx-2 px-2 transition-colors"
                        , classList [ ( "bg-base-200", proposed /= Nothing ) ]
                        ]
                        [ slider (dot param)
                            param.label
                            { min = param.min, max = param.max, step = param.step }
                            (proposed |> Maybe.map .value |> Maybe.withDefault (param.get cup))
                            (SetParam key)
                        ]
    in
    div
        [ class "relative h-screen w-screen overflow-hidden bg-base-200" ]
        [ lazy6 viewPreview cup toolpath infill spacing gesture hover
        , aside
            [ class
                "card card-sm absolute top-4 right-4 w-80 bg-base-100 border-base-300 border-2 text-base-content shadow-lg select-none resize-y overflow-auto"
            , style "transform" panelTrans
            ]
            [ div [ class "card-body gap-2" ]
                [ h2 (class "card-title justify-between cursor-move" :: dragHandle)
                    [ text "Configuration"
                    , span [ class "icon-[hugeicons--grip-vertical]" ] []
                    ]
                , viewChoosing gesture
                , profilePicker "Bottom" cup.bottom CupSetBottom
                , paramSlider "bottomRadius"
                , profilePicker "Top" cup.top CupSetTop
                , paramSlider "topRadius"
                , paramSlider "height"
                , paramSlider "fluteDepth"
                , slider (text "") "Flutes" { min = 0, max = 40, step = 1 } (toFloat cup.fluteCount) (round >> CupSetFluteCount)
                , paramSlider "ringDepth"
                , slider (text "") "Rings" { min = 0, max = 20, step = 1 } (toFloat cup.ringCount) (round >> CupSetRingCount)
                , toggle "Toolpath" toolpath SetToolpath
                , latticePicker "Infill" infill SetInfill
                , slider (text "") "Spacing" { min = 0.5, max = 5, step = 0.1 } spacing SetSpacing
                ]
            ]
        ]


{-| Shown only after an ambiguous gesture. Hovering an option shows its shape
alone; clicking it commits.
-}
viewChoosing : Gesture -> Html Msg
viewChoosing gesture =
    case gesture of
        Choosing candidates ->
            div [ class "flex flex-col gap-1 rounded-box bg-base-200 p-2" ]
                (p [ class "text-sm font-medium" ] [ text "Which one did you mean?" ]
                    :: List.filterMap choice candidates
                    ++ [ button [ type_ "button", class "btn btn-xs btn-ghost self-end", onClick DragCancel ] [ text "Cancel" ] ]
                )

        _ ->
            text ""


choice : Candidate -> Maybe (Html Msg)
choice candidate =
    findParam candidate.key
        |> Maybe.map
            (\param ->
                button
                    [ type_ "button"
                    , class "btn btn-sm justify-between opacity-70 hover:opacity-100"
                    , onClick (Choose candidate)
                    , onMouseEnter (Hover (Just param.key))
                    , onMouseLeave (Hover Nothing)
                    ]
                    [ span [ class "flex items-center gap-2" ] [ dot param, text param.label ]
                    , span [ class "text-base-content/70" ] [ text (String.fromFloat candidate.value) ]
                    ]
            )


{-| The colour of a parameter's track and silhouette on the cup.
-}
dot : Param -> Html msg
dot param =
    span [ class "inline-block size-2 rounded-full", style "background-color" param.color ] []


viewPreview : Cup -> Bool -> Maybe Lattice -> Float -> Gesture -> Maybe String -> Html Msg
viewPreview cup toolpath infill spacing gesture hover =
    let
        -- While dragging, the element draws its own previews. Afterwards,
        -- if the gesture was ambiguous, we ask it to keep showing the options.
        ghosts =
            case gesture of
                Choosing candidates ->
                    candidates

                _ ->
                    []

        encodeGhost { key, value } =
            E.object
                [ ( "key", E.string key )
                , ( "value", E.float value )
                , ( "dim", E.bool (hover /= Nothing && hover /= Just key) )
                ]

        decodeCandidates =
            D.at [ "detail", "candidates" ]
                (D.list (D.map2 Candidate (D.field "key" D.string) (D.field "value" D.float)))
    in
    node "render-preview"
        [ class "absolute inset-0 block"
        , property "params" (E.list encodeParam params)
        , property "ghosts" (E.list encodeGhost ghosts)
        , on "drag-preview" (D.map DragPreview decodeCandidates)
        , on "drag-end" (D.map DragEnd decodeCandidates)
        , on "drag-cancel" (D.succeed DragCancel)
        , property "cup" (encodeCup cup)
        , property "toolpath" (E.bool toolpath)
        , property "lattice"
            (infill
                |> Maybe.map (encodeLattice spacing)
                |> Maybe.withDefault E.null
            )
        ]
        []


slider :
    Html Msg
    -> String
    -> { min : Float, max : Float, step : Float }
    -> Float
    -> (Float -> Msg)
    -> Html Msg
slider marker name range current toMsg =
    label [ class "flex flex-col gap-1" ]
        [ span [ class "flex justify-between text-sm" ]
            [ span [ class "flex items-center gap-2" ] [ marker, text name ]
            , span [ class "text-base-content/70" ] [ text (String.fromFloat current) ]
            ]
        , input
            [ type_ "range"
            , class "range range-xs"
            , Html.Attributes.min (String.fromFloat range.min)
            , Html.Attributes.max (String.fromFloat range.max)
            , step (String.fromFloat range.step)
            , value (String.fromFloat current)
            , onInput (String.toFloat >> Maybe.withDefault current >> toMsg)
            ]
            []
        ]


toggle : String -> Bool -> (Bool -> Msg) -> Html Msg
toggle name current toMsg =
    label [ class "flex justify-between text-sm" ]
        [ text name
        , input
            [ type_ "checkbox"
            , class "toggle toggle-sm"
            , checked current
            , onCheck toMsg
            ]
            []
        ]


profilePicker : String -> Profile -> (Profile -> Msg) -> Html Msg
profilePicker name current toMsg =
    let
        option profile caption =
            button
                [ type_ "button"
                , class "btn btn-xs join-item"
                , classList [ ( "btn-active", profile == current ) ]
                , onClick (toMsg profile)
                ]
                [ text caption ]
    in
    div [ class "flex items-center justify-between text-sm" ]
        [ text name
        , div [ class "join" ]
            [ option Circle "Circle"
            , option Squircle "Squircle"
            ]
        ]


latticePicker : String -> Maybe Lattice -> (Maybe Lattice -> Msg) -> Html Msg
latticePicker name current toMsg =
    let
        option lattice caption =
            button
                [ type_ "button"
                , class "btn btn-xs join-item"
                , classList [ ( "btn-active", lattice == current ) ]
                , onClick (toMsg lattice)
                ]
                [ text caption ]
    in
    div [ class "flex items-center justify-between text-sm" ]
        [ text name
        , div [ class "join" ]
            [ option Nothing "None"
            , option (Just Grid) "Grid"
            , option (Just Honeycomb) "Honeycomb"
            ]
        ]


{-| Every draggable parameter already has a slider on this panel, so the
track labels never show "+" (which means "dropping adds a slider").
-}
encodeParam : Param -> E.Value
encodeParam param =
    E.object
        [ ( "key", E.string param.key )
        , ( "label", E.string param.label )
        , ( "hasControl", E.bool True )
        , ( "path", E.list E.string param.path )
        , ( "min", E.float param.min )
        , ( "max", E.float param.max )
        , ( "step", E.float param.step )
        , ( "color", E.string param.color )
        ]


{-| Must match `Cup` in `dsl/src/lib.rs`
-}
encodeCup : Cup -> E.Value
encodeCup cup =
    E.object
        [ ( "bottomRadius", E.float cup.bottomRadius )
        , ( "topRadius", E.float cup.topRadius )
        , ( "bottom", encodeProfile cup.bottom )
        , ( "top", encodeProfile cup.top )
        , ( "height", E.float cup.height )
        , ( "flutes"
          , E.object
                [ ( "amplitude", E.float cup.fluteAmplitude )
                , ( "count", E.int cup.fluteCount )
                ]
          )
        , ( "rings"
          , E.object
                [ ( "amplitude", E.float cup.ringAmplitude )
                , ( "count", E.int cup.ringCount )
                ]
          )
        ]


encodeProfile : Profile -> E.Value
encodeProfile profile =
    case profile of
        Circle ->
            E.string "Circle"

        Squircle ->
            E.string "Squircle"


encodeLattice : Float -> Lattice -> E.Value
encodeLattice spacing lattice =
    let
        tag =
            case lattice of
                Grid ->
                    "Grid"

                Honeycomb ->
                    "Honeycomb"
    in
    E.object [ ( tag, E.object [ ( "spacing", E.float spacing ) ] ) ]



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Draggable.subscriptions PanelDragMsg model.panelState.drag
